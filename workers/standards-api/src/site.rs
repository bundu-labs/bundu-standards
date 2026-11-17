//! The site's own routes: the pages, `/register/index.json` and the stylesheet, with the
//! headers `workers/standards/public/_headers` gave them. Platform-free, like [`crate::core`].

use crate::core::Reply;
use crate::pages::{self, CSS, CSS_PATH};

/// The security headers every page carries (from `public/_headers`), besides the content
/// security policy, which [`csp`] writes per request.
const SECURITY: [(&str, &str); 4] = [
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    ("x-frame-options", "DENY"),
    (
        "permissions-policy",
        "geolocation=(), camera=(), microphone=()",
    ),
];

/// The content security policy, with a fresh `nonce` for each response.
///
/// Our own scripts load from this origin (`'self'`) and need no nonce. The nonce is there for
/// Cloudflare's bot detection: it injects an inline script into HTML responses, and it stamps
/// that script with the nonce it reads from this header (Cloudflare docs, "JavaScript
/// Detections", "If you have a Content Security Policy"). Without one, the injected script is
/// blocked. The nonce must be unpredictable and never reused, so the Worker draws it from the
/// platform's cryptographic random source for every request.
pub fn csp(nonce: &str) -> String {
    format!(
        "default-src 'none'; script-src 'self' 'nonce-{nonce}'; style-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
    )
}

/// The answer for a site route, or `None` to hand the request to [`crate::core::route`].
pub fn reply(method: &str, path: &str, nonce: &str) -> Option<Reply> {
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
    headers.push(("content-security-policy", csp(nonce)));
    headers.push(("content-type", content_type.to_owned()));
    // The pages change only on a release, which redeploys the Worker. A page carries a
    // one-time nonce, so only the visitor's own browser may keep it; data and the stylesheet
    // may sit in shared caches.
    let scope = if content_type.starts_with("text/html") {
        "private"
    } else {
        "public"
    };
    headers.push(("cache-control", format!("{scope}, max-age=300")));
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

    const NONCE: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn the_policy_allows_our_scripts_and_the_nonce_and_nothing_inline() {
        let r = reply("GET", "/register/", NONCE).unwrap();
        let p = r.header("content-security-policy").unwrap();
        assert!(p.contains(&format!("script-src 'self' 'nonce-{NONCE}';")));
        assert!(!p.contains("unsafe-inline") && !p.contains("unsafe-eval"));
        assert!(p.starts_with("default-src 'none';"));
        assert_eq!(
            r.header("cache-control"),
            Some("private, max-age=300"),
            "no shared cache keeps a nonce"
        );
        let css = reply("GET", "/styles/site.css", NONCE).unwrap();
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
        let r = reply("GET", "/", NONCE).unwrap();
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
        let r = reply("HEAD", "/register/index.json", NONCE).unwrap();
        assert_eq!(r.header("access-control-allow-origin"), Some("*"));
        assert!(r.body.is_none());
    }

    #[test]
    fn the_stylesheet_is_served() {
        let r = reply("GET", "/styles/site.css", NONCE).unwrap();
        assert_eq!(r.header("content-type"), Some("text/css; charset=utf-8"));
        assert!(r.body.unwrap().starts_with(b"/* standards.bundu.org"));
    }

    #[test]
    fn everything_else_falls_through_to_the_file_endpoint() {
        assert!(reply("GET", "/register/latest", NONCE).is_none());
        assert!(reply("GET", "/healthz", NONCE).is_none());
        assert!(reply("POST", "/", NONCE).is_none());
        assert!(reply(
            "GET",
            "/0123456789abcdef0123456789abcdef01234567/register/register.yaml",
            NONCE
        )
        .is_none());
    }
}
