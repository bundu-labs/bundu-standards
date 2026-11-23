//! The static files of `public/` (favicons, `robots.txt`, `copy.js`, `ecosystem.jsonld`) and
//! the rules of `public/_headers`, applied to them as they always were.
//!
//! The files are compiled into the binary, so the container needs nothing but the binary.
//! `build.rs` lists `public/` (every file but `_headers`), so a file added there is served
//! with no code change. The rules are parsed from the same `_headers` file, so the security
//! headers have one source.

use crate::core::Reply;
use std::sync::OnceLock;

/// `public/_headers`, in the common `_headers` format: a path pattern on a line of
/// its own, then its headers indented beneath it. `*` in a pattern matches any run of
/// characters.
const HEADERS: &str = include_str!("../public/_headers");

// `FILES: &[(&str, &[u8])]`: the URL path and the bytes of each file in `public/`.
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// One block of `_headers`: a pattern and the headers it sets.
#[derive(Debug)]
pub struct Rule {
    pub pattern: String,
    pub headers: Vec<(&'static str, String)>,
}

/// The rules of `public/_headers`, in file order. Header names are lowercased; there are
/// a handful, parsed once, so they are leaked to be `'static` like every other header name.
pub fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| parse(HEADERS))
}

fn parse(text: &str) -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            let (name, value) = line
                .trim()
                .split_once(':')
                .unwrap_or_else(|| panic!("public/_headers: not a header: {line:?}"));
            let rule = rules
                .last_mut()
                .expect("public/_headers: a header before any path");
            let name: &'static str = Box::leak(name.trim().to_ascii_lowercase().into_boxed_str());
            rule.headers.push((name, value.trim().to_owned()));
        } else {
            rules.push(Rule {
                pattern: line.trim().to_owned(),
                headers: Vec::new(),
            });
        }
    }
    rules
}

/// Whether `pattern` matches `path`: the same text, with `*` standing for any run of characters.
fn matches(pattern: &str, path: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == path,
        Some((head, tail)) => {
            let Some(rest) = path.strip_prefix(head) else {
                return false;
            };
            (0..=rest.len())
                .filter(|i| rest.is_char_boundary(*i))
                .any(|i| matches(tail, &rest[i..]))
        }
    }
}

/// The bytes of the file at a URL path, if `public/` has one.
pub fn find(path: &str) -> Option<&'static [u8]> {
    FILES.iter().find(|(p, _)| *p == path).map(|(_, b)| *b)
}

/// The content type a static file gets: what the site answered before it became a container, so the
/// switch changes no header. `_headers` overrides it where it names one.
fn content_type(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, e)| e) {
        Some("png") => "image/png",
        Some("js") => "text/javascript",
        Some("txt") => "text/plain",
        Some("json") => "application/json",
        Some("jsonld") => "application/ld+json",
        Some("css") => "text/css",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// A validator for the bytes (FNV-1a, 128 bits would be overkill: this is a cache key, not a
/// signature), quoted as an ETag.
fn etag(body: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in body {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("\"{h:016x}\"")
}

/// Whether an `If-None-Match` header names this entity tag (or is `*`).
fn not_modified(if_none_match: Option<&str>, etag: &str) -> bool {
    if_none_match.is_some_and(|v| {
        v.split(',')
            .map(|t| t.trim().trim_start_matches("W/"))
            .any(|t| t == "*" || t == etag)
    })
}

/// The answer for a static file, or `None` when `path` is not one (or the method is not GET
/// or HEAD). Always revalidated, as the files change with a release, not on a schedule.
pub fn reply(method: &str, path: &str, if_none_match: Option<&str>) -> Option<Reply> {
    if method != "GET" && method != "HEAD" {
        return None;
    }
    let body = find(path)?;
    let tag = etag(body);
    let mut headers: Vec<(&'static str, String)> = vec![
        ("content-type", content_type(path).to_owned()),
        (
            "cache-control",
            "public, max-age=0, must-revalidate".to_owned(),
        ),
        ("etag", tag.clone()),
    ];
    for rule in rules().iter().filter(|r| matches(&r.pattern, path)) {
        for (name, value) in &rule.headers {
            match headers.iter_mut().find(|(k, _)| k == name) {
                Some(slot) => slot.1 = value.clone(),
                None => headers.push((name, value.clone())),
            }
        }
    }
    if not_modified(if_none_match, &tag) {
        headers.retain(|(k, _)| *k != "content-type");
        return Some(Reply {
            status: 304,
            headers,
            body: None,
        });
    }
    Some(Reply {
        status: 200,
        headers,
        body: (method == "GET").then(|| body.to_vec()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_of_public_is_served_but_the_headers_file() {
        for name in [
            "/robots.txt",
            "/copy.js",
            "/ecosystem.jsonld",
            "/apple-touch-icon.png",
            "/favicon-16x16.png",
        ] {
            assert!(find(name).is_some(), "{name}");
        }
        assert!(find("/_headers").is_none());
        assert!(reply("GET", "/_headers", None).is_none());
        assert!(reply("GET", "/nope.txt", None).is_none());
        assert!(reply("POST", "/robots.txt", None).is_none());
    }

    #[test]
    fn the_rules_are_parsed_in_file_order() {
        let r = rules();
        assert_eq!(r[0].pattern, "/*");
        assert!(r[0]
            .headers
            .iter()
            .any(|(k, v)| *k == "x-frame-options" && v == "DENY"));
        assert!(r.iter().any(|r| r.pattern == "/register/index.json"));
    }

    #[test]
    fn patterns_match_with_a_splat() {
        assert!(matches("/*", "/robots.txt"));
        assert!(matches("/*", "/a/b"));
        assert!(matches("/a/*/c", "/a/b/c"));
        assert!(!matches("/a/*/c", "/a/b/d"));
        assert!(matches("/robots.txt", "/robots.txt"));
        assert!(!matches("/robots.txt", "/robots.txt2"));
    }

    #[test]
    fn a_file_carries_the_security_headers_and_revalidates() {
        let r = reply("GET", "/robots.txt", None).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.header("content-type"), Some("text/plain"));
        assert_eq!(
            r.header("cache-control"),
            Some("public, max-age=0, must-revalidate")
        );
        assert_eq!(r.header("x-content-type-options"), Some("nosniff"));
        assert_eq!(r.header("referrer-policy"), Some("no-referrer"));
        assert_eq!(r.header("x-frame-options"), Some("DENY"));
        assert_eq!(
            r.header("strict-transport-security"),
            Some("max-age=31536000")
        );
        assert!(r
            .header("content-security-policy")
            .unwrap()
            .starts_with("default-src 'none'"));
        assert_eq!(r.body.as_deref(), find("/robots.txt"));
    }

    #[test]
    fn the_headers_file_sets_the_ecosystem_content_type_and_cors() {
        let r = reply("GET", "/ecosystem.jsonld", None).unwrap();
        assert_eq!(
            r.header("content-type"),
            Some("application/ld+json; charset=utf-8")
        );
        assert_eq!(r.header("access-control-allow-origin"), Some("*"));
        let robots = reply("GET", "/robots.txt", None).unwrap();
        assert_eq!(robots.header("access-control-allow-origin"), None);
    }

    #[test]
    fn head_has_no_body_and_a_matching_etag_is_a_304() {
        let head = reply("HEAD", "/copy.js", None).unwrap();
        assert_eq!(head.status, 200);
        assert!(head.body.is_none());
        let tag = head.header("etag").unwrap().to_owned();
        let same = reply("GET", "/copy.js", Some(&tag)).unwrap();
        assert_eq!(same.status, 304);
        assert!(same.body.is_none());
        assert_eq!(same.header("etag"), Some(tag.as_str()));
        let weak = reply("GET", "/copy.js", Some(&format!("W/{tag}, \"x\""))).unwrap();
        assert_eq!(weak.status, 304);
        assert_eq!(
            reply("GET", "/copy.js", Some("\"other\"")).unwrap().status,
            200
        );
    }
}
