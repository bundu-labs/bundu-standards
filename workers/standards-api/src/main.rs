//! The standards.bundu.org server: one binary, no runtime dependencies.
//!
//! Environment:
//!
//! - `LISTEN_ADDR`: where to listen, default `0.0.0.0:8080`.
//! - `PUBLIC_ORIGIN`: the origin the answers name themselves by (the URLs in
//!   `/register/latest`), default `https://standards.bundu.org`. It is set, not read from the
//!   request, so a forged `Host` header cannot put another origin into a cached answer.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, HeaderName, HeaderValue, Response, StatusCode};
use axum::Router;
use bundu_standards_api::core::Reply;
use bundu_standards_api::github::GitHub;
use bundu_standards_api::server::{self, Cache};
use std::sync::Arc;

struct App {
    origin: String,
    github: GitHub,
    cache: Cache,
}

async fn handle(State(app): State<Arc<App>>, req: Request) -> Response<Body> {
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();
    let if_none_match = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok());
    let reply = server::handle(
        &method,
        &app.origin,
        &path,
        if_none_match,
        &app.github,
        &app.cache,
    )
    .await;
    to_response(reply)
}

fn to_response(reply: Reply) -> Response<Body> {
    let mut res = Response::new(match reply.body {
        Some(body) => Body::from(body),
        None => Body::empty(),
    });
    *res.status_mut() = StatusCode::from_u16(reply.status).unwrap_or(StatusCode::BAD_GATEWAY);
    for (name, value) in reply.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            res.headers_mut().insert(name, value);
        }
    }
    res
}

async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { () = ctrl_c => {}, () = term => {} }
}

#[tokio::main]
async fn main() {
    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
    let origin = std::env::var("PUBLIC_ORIGIN")
        .unwrap_or_else(|_| "https://standards.bundu.org".to_owned())
        .trim_end_matches('/')
        .to_owned();
    let app = Arc::new(App {
        origin,
        github: GitHub::new().expect("build the HTTPS client"),
        cache: Cache::new(),
    });
    let router = Router::new().fallback(handle).with_state(app);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("listen on {addr}: {e}"));
    eprintln!("standards listening on {addr}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("serve");
}
