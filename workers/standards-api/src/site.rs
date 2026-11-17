//! The site's own routes: the pages, `/register/index.json` and the stylesheet, with the
//! headers `workers/standards/public/_headers` gave them. Platform-free, like [`crate::core`].

use crate::core::Reply;
use crate::pages::{self, CSS, CSS_PATH};

/// The security headers every page carries (from `public/_headers`).
const SECURITY: [(&str, &str); 5] = [
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    ("x-frame-options", "DENY"),
    ("permissions-policy", "geolocation=(), camera=(), microphone=()"),
    (
        "content-security-policy",
        "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
    ),
];

/// The answer for a site route, or `None` to hand the request to [`crate::core::route`].
pub fn reply(method: &str, path: &str) -> Option<Reply> {
    if method != "GET" && method != "HEAD" {
        return None;
    }
    let (content_type, body, cors) = if path == CSS_PATH {
        ("text/css; charset=utf-8", CSS.to_owned(), false)
    } else {
        let page = pages::render(path)?;
        let cors = page.content_type.starts_with("application/json");
        (page.content_type, page.body, cors)
    };
    let mut headers: Vec<(&'static str, String)> = SECURITY
        .iter()
        .map(|(k, v)| (*k, (*v).to_owned()))
        .collect();
    headers.push(("content-type", content_type.to_owned()));
    // The pages change only on a release, which redeploys the Worker.
    headers.push(("cache-control", "public, max-age=300".to_owned()));
    if cors {
        headers.push(("access-control-allow-origin", "*".to_owned()));
    }
    let body = (method == "GET").then(|| body.into_bytes());
    Some(Reply {
        status: 200,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_carry_the_security_headers() {
        let r = reply("GET", "/").unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.header("content-type"), Some("text/html; charset=utf-8"));
        assert!(r
            .header("content-security-policy")
            .unwrap()
            .contains("default-src 'none'"));
        assert_eq!(r.header("x-frame-options"), Some("DENY"));
        assert_eq!(r.header("access-control-allow-origin"), None);
    }

    #[test]
    fn the_json_is_open_to_any_origin_and_head_has_no_body() {
        let r = reply("HEAD", "/register/index.json").unwrap();
        assert_eq!(r.header("access-control-allow-origin"), Some("*"));
        assert!(r.body.is_none());
    }

    #[test]
    fn the_stylesheet_is_served() {
        let r = reply("GET", "/styles/site.css").unwrap();
        assert_eq!(r.header("content-type"), Some("text/css; charset=utf-8"));
        assert!(r.body.unwrap().starts_with(b"/* standards.bundu.org"));
    }

    #[test]
    fn everything_else_falls_through_to_the_file_endpoint() {
        assert!(reply("GET", "/register/latest").is_none());
        assert!(reply("GET", "/healthz").is_none());
        assert!(reply("POST", "/").is_none());
        assert!(reply(
            "GET",
            "/0123456789abcdef0123456789abcdef01234567/register/register.yaml"
        )
        .is_none());
    }
}
