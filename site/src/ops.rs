//! What both servers share for running in a container: the `healthcheck` subcommand the
//! Dockerfile's HEALTHCHECK runs (the runtime image is distroless, so it has no shell and no
//! curl), and the one JSON line each request leaves in the container log.
//!
//! Everything here uses the standard library only, so neither server binary gains a dependency.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// How long the health check waits to connect, and then to read, before it calls the server
/// unhealthy. Docker's own HEALTHCHECK timeout is longer, so this one answers first.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(3);

/// The longest path written to the log; a longer one is cut, so a hostile request cannot
/// fill the log.
const MAX_LOGGED_PATH: usize = 200;

/// The address the check asks: always the loopback, on the port the server listens on
/// (`LISTEN_ADDR`, default `0.0.0.0:8080`).
pub fn health_addr(listen_addr: Option<&str>) -> SocketAddr {
    let port = listen_addr
        .and_then(|a| a.rsplit_once(':'))
        .and_then(|(_, p)| p.parse::<u16>().ok())
        .unwrap_or(8080);
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// Ask `GET /healthz` at `addr`; true only for a 200 answer inside the timeout.
pub fn is_healthy(addr: SocketAddr, timeout: Duration) -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    if stream.set_read_timeout(Some(timeout)).is_err()
        || stream.set_write_timeout(Some(timeout)).is_err()
    {
        return false;
    }
    let request = "GET /healthz HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    // The status line is all that is needed: read until the first line ends.
    let mut head = Vec::new();
    let mut buf = [0u8; 256];
    while !head.contains(&b'\n') && head.len() < 1024 {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => head.extend_from_slice(&buf[..n]),
        }
    }
    let line = String::from_utf8_lossy(&head);
    let mut parts = line.lines().next().unwrap_or("").split_whitespace();
    matches!(
        (parts.next(), parts.next()),
        (Some("HTTP/1.0" | "HTTP/1.1"), Some("200"))
    )
}

/// The `healthcheck` subcommand: the process exit code (0 healthy, 1 not). The binary calls
/// this when its first argument is `healthcheck`, before starting anything else.
pub fn healthcheck_exit_code() -> i32 {
    let listen = std::env::var("LISTEN_ADDR").ok();
    i32::from(!is_healthy(health_addr(listen.as_deref()), HEALTH_TIMEOUT))
}

/// Whether the command line asks for the health check.
pub fn is_healthcheck_command(mut args: impl Iterator<Item = String>) -> bool {
    args.nth(1).as_deref() == Some("healthcheck")
}

/// One request as one JSON line: method, path (never the query string), status and duration
/// in milliseconds. Nothing about the caller: no address, no user agent, no headers.
pub fn request_log_line(method: &str, path: &str, status: u16, duration_ms: u128) -> String {
    let path: String = path.chars().take(MAX_LOGGED_PATH).collect();
    format!(
        "{{\"msg\":\"request\",\"method\":{},\"path\":{},\"status\":{status},\"duration_ms\":{duration_ms}}}",
        json_string(method),
        json_string(&path),
    )
}

/// A JSON string literal. Only the characters JSON requires escaping are escaped.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Write the request's line to standard output, where the container log collects it.
pub fn log_request(method: &str, path: &str, status: u16, duration: Duration) {
    println!(
        "{}",
        request_log_line(method, path, status, duration.as_millis())
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, ToSocketAddrs};
    use std::thread;

    /// A one-shot server that answers the first connection with `response`, then returns.
    fn serve_once(response: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let _ = s.read(&mut buf);
                let _ = s.write_all(response.as_bytes());
            }
        });
        addr
    }

    fn resolve(addr: &str) -> SocketAddr {
        addr.to_socket_addrs().unwrap().next().unwrap()
    }

    #[test]
    fn a_200_is_healthy() {
        let addr = serve_once("HTTP/1.1 200 OK\r\ncontent-length: 15\r\n\r\n{\"status\":\"ok\"}");
        assert!(is_healthy(addr, Duration::from_secs(2)));
    }

    #[test]
    fn anything_but_200_is_unhealthy() {
        for r in [
            "HTTP/1.1 500 Internal Server Error\r\n\r\n",
            "HTTP/1.1 404 Not Found\r\n\r\n",
            "HTTP/1.1 2000 Odd\r\n\r\n",
            "garbage\r\n\r\n",
            "",
        ] {
            assert!(!is_healthy(serve_once(r), Duration::from_secs(2)), "{r:?}");
        }
    }

    #[test]
    fn nothing_listening_is_unhealthy() {
        let addr = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap()
        };
        assert!(!is_healthy(addr, Duration::from_secs(1)));
    }

    #[test]
    fn a_server_that_never_answers_is_unhealthy_within_the_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let hold = thread::spawn(move || {
            let c = listener.accept();
            thread::sleep(Duration::from_secs(2));
            drop(c);
        });
        let started = std::time::Instant::now();
        assert!(!is_healthy(addr, Duration::from_millis(300)));
        assert!(started.elapsed() < Duration::from_millis(1500));
        hold.join().unwrap();
    }

    #[test]
    fn the_address_is_the_loopback_on_the_listen_port() {
        assert_eq!(health_addr(None), resolve("127.0.0.1:8080"));
        assert_eq!(health_addr(Some("0.0.0.0:8080")), resolve("127.0.0.1:8080"));
        assert_eq!(health_addr(Some("0.0.0.0:9000")), resolve("127.0.0.1:9000"));
        assert_eq!(health_addr(Some("[::]:7000")), resolve("127.0.0.1:7000"));
        assert_eq!(health_addr(Some("nonsense")), resolve("127.0.0.1:8080"));
    }

    #[test]
    fn the_subcommand_is_recognised_only_as_the_first_argument() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(is_healthcheck_command(
            args(&["/server", "healthcheck"]).into_iter()
        ));
        assert!(!is_healthcheck_command(args(&["/server"]).into_iter()));
        assert!(!is_healthcheck_command(
            args(&["/server", "serve"]).into_iter()
        ));
    }

    #[test]
    fn a_request_line_is_one_json_object_with_no_personal_fields() {
        let line = request_log_line("GET", "/Family", 200, 3);
        let v: serde_json_like::Parsed = serde_json_like::parse(&line);
        assert_eq!(v.keys, ["msg", "method", "path", "status", "duration_ms"]);
        assert!(!line.contains('\n'));
        assert_eq!(
            line,
            r#"{"msg":"request","method":"GET","path":"/Family","status":200,"duration_ms":3}"#
        );
    }

    #[test]
    fn the_path_is_escaped_and_bounded() {
        let line = request_log_line("GET", "/a\"b\\c\nd", 404, 0);
        assert!(line.contains(r#""path":"/a\"b\\c\nd""#));
        assert!(!line.contains('\n'));
        let long = format!("/{}", "x".repeat(5000));
        assert!(request_log_line("GET", &long, 200, 0).len() < 400);
    }

    /// A minimal check that a line is a flat JSON object and what its keys are, so the tests
    /// need no JSON crate here.
    mod serde_json_like {
        pub struct Parsed {
            pub keys: Vec<String>,
        }
        pub fn parse(line: &str) -> Parsed {
            assert!(line.starts_with('{') && line.ends_with('}'));
            let mut keys = Vec::new();
            let mut depth_in_string = false;
            let mut cur = String::new();
            let mut expect_key = true;
            let mut escaped = false;
            for c in line[1..line.len() - 1].chars() {
                if depth_in_string {
                    if escaped {
                        cur.push(c);
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        depth_in_string = false;
                        if expect_key {
                            keys.push(std::mem::take(&mut cur));
                            expect_key = false;
                        } else {
                            cur.clear();
                        }
                    } else {
                        cur.push(c);
                    }
                } else if c == '"' {
                    depth_in_string = true;
                } else if c == ',' {
                    expect_key = true;
                }
            }
            Parsed { keys }
        }
    }
}
