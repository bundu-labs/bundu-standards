//! The site's own routes: the pages, `/register/index.json` and the stylesheet, with the
//! headers `public/_headers` gives the static files. Server-free, like [`crate::core`].

use crate::core::Reply;
use crate::pages::{self, CSS, CSS_PATH};

/// The security headers every page carries (the same as `public/_headers`), besides the
/// content security policy, which is [`CSP`].
const SECURITY: [(&str, &str); 4] = [
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    ("x-frame-options", "DENY"),
    (
        "permissions-policy",
        "geolocation=(), camera=(), microphone=()",
    ),
];

/// The content security policy, the same as the `/*` rule of `public/_headers`: our own
/// scripts and styles load from this origin and nothing is inline. A test holds the two equal.
pub const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

/// The answer for a site route, or `None` to hand the request to [`crate::core::route`].
pub fn reply(method: &str, path: &str) -> Option<Reply> {
    if method != "GET" && method != "HEAD" {
        return None;
    }
    let (content_type, body, cors) = if path == CSS_PATH {
        ("text/css; charset=utf-8", CSS.to_owned(), false)
    } else {
        let page = pages::render(path)?;
        // Data is open to any origin: the JSON index and our standards' files.
        let cors = !page.content_type.starts_with("text/html");
        (page.content_type, page.body, cors)
    };
    let mut headers: Vec<(&'static str, String)> = SECURITY
        .iter()
        .map(|(k, v)| (*k, (*v).to_owned()))
        .collect();
    headers.push(("content-security-policy", CSP.to_owned()));
    headers.push(("content-type", content_type.to_owned()));
    // The pages change only on a release, which redeploys the server.
    headers.push(("cache-control", "public, max-age=300".to_owned()));
    if path == CSS_PATH {
        headers.push(("etag", bundu_site::CSS_ETAG.to_owned()));
    }
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
    fn the_policy_is_the_one_in_the_headers_file_and_allows_nothing_inline() {
        let r = reply("GET", "/register/").unwrap();
        let p = r.header("content-security-policy").unwrap();
        let headers = crate::assets::rules();
        let all = headers.iter().find(|r| r.pattern == "/*").unwrap();
        let file = all
            .headers
            .iter()
            .find(|(k, _)| *k == "content-security-policy")
            .unwrap();
        assert_eq!(p, file.1, "public/_headers and CSP must agree");
        assert!(!p.contains("unsafe-inline") && !p.contains("unsafe-eval"));
        assert!(p.starts_with("default-src 'none';"));
        assert_eq!(r.header("cache-control"), Some("public, max-age=300"));
        let css = reply("GET", "/styles/site.css").unwrap();
        assert_eq!(css.header("cache-control"), Some("public, max-age=300"));
        // The pages themselves carry no inline script that runs: their only inline scripts are
        // data (JSON-LD and the copy control's JSON), which a browser never executes.
        let body = String::from_utf8(r.body.unwrap()).unwrap();
        for tag in body
            .match_indices("<script")
            .map(|(i, _)| &body[i..i + body[i..].find('>').unwrap()])
        {
            assert!(
                tag.contains("src=")
                    || tag.contains("application/ld+json")
                    || tag.contains("application/json"),
                "inline script: {tag}"
            );
        }
    }

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
        assert_eq!(r.header("etag"), Some(bundu_site::CSS_ETAG));
        assert!(r.header("etag").unwrap().starts_with('"'));
        let body = String::from_utf8(r.body.unwrap()).unwrap();
        assert!(body.starts_with(":root{"), "minified, token layer first");
        assert!(body.contains("var(--target-floor)"));
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
