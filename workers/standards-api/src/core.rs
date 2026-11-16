//! The file endpoint of standards.bundu.org, without the platform.
//!
//! It holds no data and no secret. It serves four files of `register/` from the public
//! repository `bundu-labs/bundu-standards` at a commit the caller names:
//!
//! ```text
//! /<40-hex commit>/register/register.yaml
//! /<40-hex commit>/register/REGISTER.md
//! /<40-hex commit>/register/seed/integrations.standards.json
//! /<40-hex commit>/register/tools/register.py
//! /register/latest            the head of `main`, as JSON
//! /healthz
//! ```
//!
//! The same behaviour as `workers/standards/src/lib/files.js`, which it replaces: the path after
//! the host is the one raw.githubusercontent.com uses, a commit's files are cached for a year,
//! and nothing outside the four files is fetched (this is not an open proxy).

use serde_json::json;

pub const REPO: &str = "bundu-labs/bundu-standards";
pub const RAW: &str = "https://raw.githubusercontent.com/bundu-labs/bundu-standards";
pub const LATEST_URL: &str = "https://api.github.com/repos/bundu-labs/bundu-standards/commits/main";
/// Git's own ref advertisement. Not the REST API, so not held to its limit of 60
/// unauthenticated requests an hour per IP, which Cloudflare's shared egress runs out of.
pub const REFS_URL: &str =
    "https://github.com/bundu-labs/bundu-standards.git/info/refs?service=git-upload-pack";
pub const USER_AGENT: &str = "standards.bundu.org";

/// The only files served, and their content types. Anything else is a 404.
pub const FILES: [(&str, &str); 4] = [
    ("register.yaml", "application/yaml; charset=utf-8"),
    ("REGISTER.md", "text/markdown; charset=utf-8"),
    (
        "seed/integrations.standards.json",
        "application/json; charset=utf-8",
    ),
    ("tools/register.py", "text/x-python; charset=utf-8"),
];

const COMMON: [(&str, &str); 3] = [
    ("access-control-allow-origin", "*"),
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
];

/// An upstream answer: its status and body. A network failure is `Err`.
pub struct Upstream {
    pub status: u16,
    pub body: Vec<u8>,
}

/// How the endpoint reaches GitHub. The Worker uses `fetch`; the tests use a fake.
#[allow(async_fn_in_trait)]
pub trait Fetch {
    async fn get(&self, url: &str, accept: Option<&str>) -> Result<Upstream, ()>;
}

/// A response, independent of the platform.
#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: Option<Vec<u8>>,
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn with_common(status: u16, extra: &[(&'static str, String)], body: Option<Vec<u8>>) -> Self {
        let mut headers: Vec<(&'static str, String)> =
            COMMON.iter().map(|(k, v)| (*k, v.to_string())).collect();
        headers.extend_from_slice(extra);
        Reply {
            status,
            headers,
            body,
        }
    }
}

/// RFC 9457 problem details.
pub fn problem(status: u16, title: &str, detail: &str, instance: &str) -> Reply {
    let body = json!({
        "type": "about:blank",
        "title": title,
        "status": status,
        "detail": detail,
        "instance": instance,
    });
    Reply::with_common(
        status,
        &[
            ("content-type", "application/problem+json".into()),
            ("cache-control", "no-store".into()),
        ],
        Some(body.to_string().into_bytes()),
    )
}

fn json_reply(body: serde_json::Value, cache: &str) -> Reply {
    Reply::with_common(
        200,
        &[
            ("content-type", "application/json; charset=utf-8".into()),
            ("cache-control", cache.into()),
        ],
        Some(body.to_string().into_bytes()),
    )
}

fn is_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `/<sha>/register/<file>` split into its commit and file, if the path has that shape.
fn sha_path(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix('/')?;
    let (sha, rest) = rest.split_at_checked(40)?;
    let file = rest.strip_prefix("/register/")?;
    (is_sha(sha) && !file.is_empty()).then_some((sha, file))
}

/// The cache key ignores the query string: `?x=1`, `?x=2` must not each reach the upstream.
pub fn cache_key(origin: &str, path: &str) -> String {
    format!("{origin}{path}")
}

/// Answer one request for /healthz, /register/latest or /<commit>/register/<file>.
pub async fn route(method: &str, origin: &str, path: &str, fetch: &impl Fetch) -> Reply {
    if method == "OPTIONS" {
        return Reply::with_common(
            204,
            &[
                ("access-control-allow-methods", "GET, HEAD, OPTIONS".into()),
                ("access-control-max-age", "86400".into()),
            ],
            None,
        );
    }
    if method != "GET" && method != "HEAD" {
        return problem(
            405,
            "Method not allowed",
            "This endpoint is read-only.",
            path,
        );
    }

    if path == "/healthz" || path == "/healthz/" {
        return json_reply(json!({ "status": "ok" }), "no-store");
    }
    if path == "/register/latest" || path == "/register/latest/" {
        return latest(origin, path, fetch).await;
    }

    let Some((sha, file)) = sha_path(path) else {
        return problem(404, "Not found", "No such resource.", path);
    };
    let Some((_, content_type)) = FILES.iter().find(|(f, _)| *f == file) else {
        return problem(
            404,
            "Not found",
            "That file is not part of the register.",
            path,
        );
    };

    let upstream = match fetch
        .get(&format!("{RAW}/{sha}/register/{file}"), None)
        .await
    {
        Ok(u) => u,
        Err(()) => {
            return problem(
                502,
                "Bad gateway",
                "The repository could not be reached.",
                path,
            )
        }
    };
    if upstream.status == 404 {
        return problem(
            404,
            "Not found",
            "That commit or file does not exist.",
            path,
        );
    }
    if !(200..300).contains(&upstream.status) {
        let detail = format!("The repository answered {}.", upstream.status);
        return problem(502, "Bad gateway", &detail, path);
    }

    Reply::with_common(
        200,
        &[
            ("content-type", (*content_type).into()),
            // A commit never changes.
            (
                "cache-control",
                "public, max-age=31536000, immutable".into(),
            ),
            ("etag", format!("\"{sha}:{file}\"")),
        ],
        (method != "HEAD").then_some(upstream.body),
    )
}

/// The head of `main` from the REST API: the status it answered (0 for a network failure) and
/// the commit, if it gave one.
async fn head_from_api(fetch: &impl Fetch) -> (u16, Option<String>) {
    match fetch
        .get(LATEST_URL, Some("application/vnd.github+json"))
        .await
    {
        Err(()) => (0, None),
        Ok(u) if !(200..300).contains(&u.status) => (u.status, None),
        Ok(u) => {
            let sha = serde_json::from_slice::<serde_json::Value>(&u.body)
                .ok()
                .and_then(|v| v.get("sha")?.as_str().map(str::to_owned))
                .filter(|s| is_sha(s));
            (u.status, sha)
        }
    }
}

/// The head of `main` from git's ref advertisement: a pkt-line `<4 hex><sha> refs/heads/main`.
async fn head_from_refs(fetch: &impl Fetch) -> Option<String> {
    let u = fetch.get(REFS_URL, None).await.ok()?;
    if !(200..300).contains(&u.status) {
        return None;
    }
    let text = String::from_utf8(u.body).ok()?;
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        // A line counts only when a newline follows it, as in the advertisement.
        if lines.peek().is_none() {
            break;
        }
        let Some(rest) = line.strip_suffix(" refs/heads/main") else {
            continue;
        };
        if rest.len() == 44
            && rest[..4].bytes().all(|b| b.is_ascii_hexdigit())
            && is_sha(&rest[4..])
        {
            return Some(rest[4..].to_owned());
        }
    }
    None
}

async fn latest(origin: &str, path: &str, fetch: &impl Fetch) -> Reply {
    let (api_status, api_ref) = head_from_api(fetch).await;
    let head = match api_ref {
        Some(r) => Some(r),
        None => head_from_refs(fetch).await,
    };
    let Some(sha) = head else {
        let why = if api_status != 0 && api_status != 200 {
            format!("The repository answered {api_status}.")
        } else {
            "The repository answer had no commit.".to_owned()
        };
        return problem(502, "Bad gateway", &why, path);
    };
    let files: Vec<_> = FILES
        .iter()
        .map(|(f, _)| json!({ "path": f, "url": format!("{origin}/{sha}/register/{f}") }))
        .collect();
    json_reply(
        json!({ "ref": sha, "branch": "main", "files": files }),
        "public, max-age=300",
    )
}

#[cfg(test)]
mod tests {
    //! The same cases as workers/standards/test/files.test.js.
    use super::*;
    use pollster::block_on;
    use std::cell::RefCell;

    const SHA: &str = "e15535a4fbaab894167cc87644baa4168b5f8150";
    const ORIGIN: &str = "https://standards.bundu.org";

    /// A fake GitHub: answers by URL, and records every URL asked.
    struct Fake<F: Fn(&str) -> Result<Upstream, ()>> {
        answer: F,
        asked: RefCell<Vec<String>>,
    }
    impl<F: Fn(&str) -> Result<Upstream, ()>> Fetch for Fake<F> {
        async fn get(&self, url: &str, _accept: Option<&str>) -> Result<Upstream, ()> {
            self.asked.borrow_mut().push(url.to_owned());
            (self.answer)(url)
        }
    }
    fn fake<F: Fn(&str) -> Result<Upstream, ()>>(answer: F) -> Fake<F> {
        Fake {
            answer,
            asked: RefCell::new(Vec::new()),
        }
    }
    fn ok(body: &str, status: u16) -> Result<Upstream, ()> {
        Ok(Upstream {
            status,
            body: body.as_bytes().to_vec(),
        })
    }
    fn never() -> Fake<impl Fn(&str) -> Result<Upstream, ()>> {
        fake(|u| panic!("must not reach the upstream: {u}"))
    }
    fn get(path: &str, f: &impl Fetch) -> Reply {
        block_on(route("GET", ORIGIN, path, f))
    }
    fn body_json(r: &Reply) -> serde_json::Value {
        serde_json::from_slice(r.body.as_deref().unwrap()).unwrap()
    }

    #[test]
    fn serves_a_register_file_at_a_commit_immutable_and_open_to_any_origin() {
        let f = fake(|_| ok("standards: []", 200));
        let res = get(&format!("/{SHA}/register/register.yaml"), &f);
        assert_eq!(res.status, 200);
        assert_eq!(
            f.asked.borrow().as_slice(),
            [format!("{RAW}/{SHA}/register/register.yaml")]
        );
        assert_eq!(res.body.as_deref(), Some(b"standards: []".as_slice()));
        assert!(res.header("cache-control").unwrap().contains("immutable"));
        assert_eq!(res.header("access-control-allow-origin"), Some("*"));
        assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
        assert!(res.header("content-type").unwrap().contains("yaml"));
    }

    #[test]
    fn serves_every_allowed_file_with_its_own_content_type() {
        for (file, ty) in FILES {
            let res = get(&format!("/{SHA}/register/{file}"), &fake(|_| ok("x", 200)));
            assert_eq!(res.status, 200, "{file}");
            assert_eq!(res.header("content-type"), Some(ty), "{file}");
        }
    }

    #[test]
    fn is_not_an_open_proxy() {
        let short = &SHA[..7];
        let upper = SHA.to_uppercase();
        for path in [
            format!("/{SHA}/register/../AGENTS.md"),
            format!("/{SHA}/register/%2e%2e/AGENTS.md"),
            format!("/{SHA}/register/tools/test_register.py"),
            format!("/{SHA}/register/"),
            format!("/{SHA}/AGENTS.md"),
            "/main/register/register.yaml".to_owned(),
            format!("/{short}/register/register.yaml"),
            format!("/{upper}/register/register.yaml"),
            format!("/{SHA}/register/__proto__"),
            format!("/{SHA}/register/constructor"),
            "/etc/passwd".to_owned(),
        ] {
            let res = get(&path, &never());
            assert_eq!(res.status, 404, "{path}");
            assert_eq!(
                res.header("content-type"),
                Some("application/problem+json"),
                "{path}"
            );
        }
    }

    #[test]
    fn a_missing_commit_is_a_404_an_upstream_failure_a_502() {
        let path = format!("/{SHA}/register/register.yaml");
        assert_eq!(get(&path, &fake(|_| ok("nope", 404))).status, 404);
        assert_eq!(get(&path, &fake(|_| ok("err", 503))).status, 502);
        let thrown = get(&path, &fake(|_| Err(())));
        assert_eq!(thrown.status, 502);
        assert_eq!(thrown.header("cache-control"), Some("no-store"));
        let body = body_json(&thrown);
        assert_eq!(body["status"], 502);
        assert_eq!(body["instance"], path);
    }

    #[test]
    fn head_has_the_headers_and_no_body() {
        let res = block_on(route(
            "HEAD",
            ORIGIN,
            &format!("/{SHA}/register/REGISTER.md"),
            &fake(|_| ok("body", 200)),
        ));
        assert_eq!(res.status, 200);
        assert!(res.body.is_none());
    }

    #[test]
    fn read_only_other_methods_refused_options_answers_the_preflight() {
        for m in ["POST", "PUT", "DELETE", "PATCH"] {
            let res = block_on(route(
                m,
                ORIGIN,
                &format!("/{SHA}/register/register.yaml"),
                &fake(|_| ok("x", 200)),
            ));
            assert_eq!(res.status, 405, "{m}");
        }
        let pre = block_on(route("OPTIONS", ORIGIN, "/register/latest", &never()));
        assert_eq!(pre.status, 204);
        assert!(pre
            .header("access-control-allow-methods")
            .unwrap()
            .contains("GET"));
    }

    #[test]
    fn latest_names_the_head_of_main_and_the_urls_of_the_four_files() {
        let f = fake(|_| ok(&format!(r#"{{"sha":"{SHA}"}}"#), 200));
        let res = get("/register/latest", &f);
        assert_eq!(f.asked.borrow().as_slice(), [LATEST_URL]);
        let body = body_json(&res);
        assert_eq!(body["ref"], SHA);
        assert_eq!(body["branch"], "main");
        assert_eq!(body["files"].as_array().unwrap().len(), FILES.len());
        assert_eq!(
            body["files"][0]["url"],
            format!("https://standards.bundu.org/{SHA}/register/register.yaml")
        );
        assert_eq!(res.header("cache-control"), Some("public, max-age=300"));
    }

    #[test]
    fn latest_refuses_an_answer_that_is_not_a_commit() {
        for bad in [r#"{"sha":"not-a-sha"}"#, "{}", "not json"] {
            // The refs fallback answers nothing useful either.
            let f = fake(move |u| {
                if u == LATEST_URL {
                    ok(bad, 200)
                } else {
                    ok("0000", 200)
                }
            });
            assert_eq!(get("/register/latest", &f).status, 502, "{bad}");
        }
        let f = fake(|u| {
            if u == LATEST_URL {
                ok("{}", 403)
            } else {
                ok("0000", 200)
            }
        });
        assert_eq!(get("/register/latest", &f).status, 502);
    }

    #[test]
    fn latest_falls_back_to_the_ref_advertisement_when_the_api_refuses() {
        let a = "a".repeat(40);
        let b = "b".repeat(40);
        let refs = format!(
            "001e# service=git-upload-pack\n0000\
             0155{a} HEAD\0multi_ack symref=HEAD:refs/heads/main\n\
             003f{SHA} refs/heads/main\n\
             0044{b} refs/heads/staging\n0000"
        );
        let f = fake(move |u| {
            if u == LATEST_URL {
                ok("{}", 403)
            } else {
                ok(&refs, 200)
            }
        });
        let res = get("/register/latest", &f);
        assert_eq!(f.asked.borrow().as_slice(), [LATEST_URL, REFS_URL]);
        assert_eq!(res.status, 200);
        assert_eq!(body_json(&res)["ref"], SHA);
    }

    #[test]
    fn latest_is_a_502_when_neither_names_main() {
        let f = fake(|u| {
            if u == LATEST_URL {
                ok("{}", 403)
            } else {
                ok("0000", 200)
            }
        });
        let res = get("/register/latest", &f);
        assert_eq!(res.status, 502);
        assert!(body_json(&res)["detail"].as_str().unwrap().contains("403"));
    }

    #[test]
    fn health() {
        assert_eq!(get("/healthz", &never()).status, 200);
    }

    #[test]
    fn the_cache_key_ignores_the_query_string() {
        // The Worker passes only the path, so `?a=1` and `?a=2` share one key.
        let key = cache_key(ORIGIN, "/register/latest");
        assert_eq!(key, "https://standards.bundu.org/register/latest");
        assert!(!key.contains('?'));
    }
}
