//! The OpenAPI 3.0 description of standards.bundu.org, served at `/openapi.json` and
//! `/.well-known/openapi.json`.
//!
//! The document is the committed file `workers/standards-api/openapi.json`, built into the
//! server. It is what API clients and schema validators read, so it must name every
//! route the server answers and nothing it does not. The tests below hold it to the router:
//! a route the server serves that the document leaves out fails them, and so does a path in the
//! document that the server does not serve.

/// The OpenAPI document, as committed.
pub const OPENAPI: &str = include_str!("../openapi.json");

#[cfg(test)]
mod tests {
    use super::OPENAPI;
    use crate::core::{self, Fetch, Upstream};
    use crate::pages::{DOCUMENTS, REGISTER};
    use crate::register::SITE;
    use crate::site;
    use pollster::block_on;
    use serde_json::Value;
    use std::collections::{BTreeSet, VecDeque};

    const SHA: &str = "e15535a4fbaab894167cc87644baa4168b5f8150";
    /// The static files the server answers before anything else (`public/`).
    const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/public");
    /// The router's source: every path literal in it that the server serves must be in the
    /// document, linked from a page or not.
    const SOURCES: [&str; 3] = [
        include_str!("core.rs"),
        include_str!("site.rs"),
        include_str!("pages.rs"),
    ];

    /// GitHub, answering every request with a commit, so the file endpoint's routes answer 200.
    struct GitHub;
    impl Fetch for GitHub {
        async fn get(&self, _url: &str, _accept: Option<&str>) -> Result<Upstream, ()> {
            Ok(Upstream {
                status: 200,
                body: format!(r#"{{"sha":"{SHA}"}}"#).into_bytes(),
            })
        }
    }

    fn doc() -> Value {
        serde_json::from_str(OPENAPI).expect("openapi.json is JSON")
    }

    /// How the deployed server answers a GET for `path`: the content type of a 200, or `None`.
    /// Static assets first, as the server does; then the site; then the file endpoint.
    fn serves(path: &str) -> Option<String> {
        if crate::assets::find(path).is_some() {
            return Some("(static asset)".to_owned());
        }
        if let Some(r) = site::reply("GET", path) {
            return (r.status == 200).then(|| r.header("content-type").unwrap().to_owned());
        }
        let r = block_on(core::route("GET", SITE, path, &GitHub));
        (r.status == 200).then(|| r.header("content-type").unwrap().to_owned())
    }

    /// Whether a concrete path matches a path template of the document.
    fn matches(template: &str, path: &str) -> bool {
        let t: Vec<&str> = template.split('/').collect();
        let p: Vec<&str> = path.split('/').collect();
        t.len() == p.len()
            && t.iter()
                .zip(&p)
                .all(|(t, p)| *t == *p || (t.starts_with('{') && t.ends_with('}') && !p.is_empty()))
    }

    /// The document's path covering `path`. The server treats a trailing slash as optional,
    /// so either form counts; the document lists the canonical one.
    fn covered<'a>(templates: &'a [String], path: &str) -> Option<&'a str> {
        let other = match path.strip_suffix('/') {
            Some(p) if !p.is_empty() => p.to_owned(),
            Some(_) => path.to_owned(),
            None => format!("{path}/"),
        };
        templates
            .iter()
            .find(|t| matches(t, path) || matches(t, &other))
            .map(String::as_str)
    }

    fn templates() -> Vec<String> {
        doc()["paths"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// Same-origin links in a page (`href` and `src`), as paths, resolved against `base`.
    fn links(html: &str, base: &str) -> Vec<String> {
        let mut out = Vec::new();
        for attr in ["href=\"", "src=\""] {
            for (i, _) in html.match_indices(attr) {
                let rest = &html[i + attr.len()..];
                let raw = &rest[..rest.find('"').unwrap_or(0)];
                let raw = raw.split(['#', '?']).next().unwrap_or("");
                let path = if let Some(p) = raw.strip_prefix(SITE) {
                    p.to_owned()
                } else if raw.starts_with("//") || raw.contains(':') || raw.is_empty() {
                    continue;
                } else if raw.starts_with('/') {
                    raw.to_owned()
                } else {
                    let dir = &base[..=base.rfind('/').unwrap()];
                    format!("{dir}{}", raw.trim_start_matches("./"))
                };
                out.push(normalise(&path));
            }
        }
        out
    }

    /// A path with its `.` and `..` segments resolved, as a browser does (RFC 3986, 5.2.4).
    fn normalise(path: &str) -> String {
        let mut out: Vec<&str> = Vec::new();
        let segments: Vec<&str> = path.split('/').skip(1).collect();
        for (i, s) in segments.iter().enumerate() {
            let last = i + 1 == segments.len();
            match *s {
                "." | ".." => {
                    if *s == ".." {
                        out.pop();
                    }
                    if last {
                        out.push("");
                    }
                }
                s => out.push(s),
            }
        }
        format!("/{}", out.join("/"))
    }

    /// Every path the server serves that we can name: the routes its data produces, every path
    /// literal in its source that it answers, the static assets, and everything a page links to.
    fn served_paths() -> BTreeSet<String> {
        let mut seeds: Vec<String> = vec!["/".into(), format!("/{SHA}/register/register.yaml")];
        seeds.extend(
            core::FILES
                .iter()
                .map(|(f, _)| format!("/{SHA}/register/{f}")),
        );
        seeds.extend(
            REGISTER
                .standards
                .iter()
                .map(|s| format!("/register/{}/", s.id)),
        );
        for (slug, d) in DOCUMENTS.iter() {
            seeds.push(format!("/{slug}/"));
            seeds.extend(
                d["files"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|f| format!("/{slug}/{f}")),
            );
        }
        for src in SOURCES {
            for lit in src.split('"').skip(1).step_by(2) {
                if lit.starts_with('/') && !lit.contains(['{', ' ', '\\']) {
                    seeds.push(lit.to_owned());
                }
            }
        }
        for e in std::fs::read_dir(ASSETS).unwrap().flatten() {
            seeds.push(format!("/{}", e.file_name().to_string_lossy()));
        }

        let mut seen = BTreeSet::new();
        let mut broken = Vec::new();
        let mut queue: VecDeque<(String, bool)> = seeds.into_iter().map(|s| (s, false)).collect();
        while let Some((path, linked)) = queue.pop_front() {
            if seen.contains(&path) {
                continue;
            }
            let Some(ty) = serves(&path) else {
                // A path literal in the source need not be a route (tests name paths that must
                // 404); a link on a page must lead somewhere.
                if linked {
                    broken.push(path);
                }
                continue;
            };
            seen.insert(path.clone());
            if ty.starts_with("text/html") {
                let html = site::reply("GET", &path).unwrap().body.unwrap();
                for l in links(&String::from_utf8(html).unwrap(), &path) {
                    queue.push_back((l, true));
                }
            }
        }
        assert!(
            broken.is_empty(),
            "pages link to paths nothing serves: {broken:?}"
        );
        seen
    }

    #[test]
    fn every_route_the_worker_serves_is_in_the_document() {
        let templates = templates();
        let served = served_paths();
        assert!(
            served.len() > REGISTER.standards.len(),
            "the crawl found the routes"
        );
        let missing: Vec<_> = served
            .iter()
            .filter(|p| covered(&templates, p).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "served by the server but not in openapi.json: {missing:?}"
        );
    }

    #[test]
    fn the_check_catches_a_route_left_out() {
        let served = served_paths();
        for gone in [
            "/accessibility/",
            "/register/{id}/",
            "/.well-known/openapi.json",
        ] {
            let templates: Vec<String> = templates().into_iter().filter(|t| t != gone).collect();
            assert!(
                served.iter().any(|p| covered(&templates, p).is_none()),
                "{gone}"
            );
        }
    }

    #[test]
    fn every_path_in_the_document_is_served_with_its_content_type() {
        let first = &REGISTER.standards[0].id;
        let doc = doc();
        for (template, item) in doc["paths"].as_object().unwrap() {
            let path = template.replace("{id}", first).replace("{commit}", SHA);
            assert!(!path.contains('{'), "{template}: an unknown path parameter");
            let ty = serves(&path).unwrap_or_else(|| panic!("{template}: not served"));
            let content = item["get"]["responses"]["200"]["content"]
                .as_object()
                .unwrap_or_else(|| panic!("{template}: no 200 content"));
            assert_eq!(content.len(), 1, "{template}: one content type");
            let declared = content.keys().next().unwrap();
            if ty != "(static asset)" {
                assert_eq!(*declared, ty, "{template}: content type");
            }
        }
    }

    /// The keys of an object, and the `required` list of a schema in the document.
    fn keys(v: &Value) -> BTreeSet<String> {
        v.as_object().unwrap().keys().cloned().collect()
    }
    fn required(doc: &Value, schema: &str) -> BTreeSet<String> {
        let s = &doc["components"]["schemas"][schema];
        let req: BTreeSet<String> = s["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        // Every schema the router's own JSON is checked against is closed: required is all.
        assert_eq!(s["additionalProperties"], false, "{schema} is closed");
        assert_eq!(
            req,
            keys(&s["properties"]),
            "{schema}: every field required"
        );
        req
    }

    #[test]
    fn the_routers_json_has_exactly_the_described_fields() {
        let doc = doc();
        let body = |path: &str| -> Value {
            let r = match site::reply("GET", path) {
                Some(r) => r,
                None => block_on(core::route("GET", SITE, path, &GitHub)),
            };
            serde_json::from_slice(r.body.as_deref().unwrap()).unwrap()
        };
        let index = body("/register/index.json");
        assert_eq!(keys(&index), required(&doc, "RegisterIndex"));
        assert_eq!(keys(&index["count"]), required(&doc, "Counts"));
        for s in index["standards"].as_array().unwrap() {
            assert_eq!(keys(s), required(&doc, "RegisterEntry"), "{}", s["id"]);
        }
        let latest = body("/register/latest");
        assert_eq!(keys(&latest), required(&doc, "Latest"));
        assert_eq!(keys(&body("/healthz")), required(&doc, "Health"));
        let files = &doc["components"]["schemas"]["Latest"]["properties"]["files"];
        let described: Vec<&str> = files["items"]["properties"]["path"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let served: Vec<&str> = core::FILES.iter().map(|(f, _)| *f).collect();
        assert_eq!(described, served);
        assert_eq!(files["minItems"], core::FILES.len());
    }

    #[test]
    fn only_get_and_head_are_described() {
        let doc = doc();
        for (template, item) in doc["paths"].as_object().unwrap() {
            let methods: BTreeSet<&str> = item
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .filter(|k| *k != "parameters")
                .collect();
            assert_eq!(
                methods,
                BTreeSet::from(["get", "head"]),
                "{template}: GET and HEAD only"
            );
        }
    }

    #[test]
    fn it_is_openapi_3_0_for_this_host() {
        let doc = doc();
        assert_eq!(doc["openapi"], "3.0.3", "schema validators take 3.0.x");
        assert_eq!(doc["servers"][0]["url"], SITE);
        assert_eq!(doc["servers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn the_id_parameter_admits_every_standard_in_the_register() {
        let doc = doc();
        let pattern = &doc["components"]["parameters"]["StandardId"]["schema"]["pattern"];
        assert_eq!(pattern, "^[a-z0-9][a-z0-9_]*$");
        let ok = |id: &str| {
            id.bytes()
                .next()
                .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        };
        for s in &REGISTER.standards {
            assert!(ok(&s.id), "register id {} does not match {pattern}", s.id);
        }
    }

    #[test]
    fn every_reference_resolves() {
        let doc = doc();
        fn walk(v: &Value, doc: &Value, at: &str) {
            match v {
                Value::Object(m) => {
                    if let Some(Value::String(r)) = m.get("$ref") {
                        let ptr = r.strip_prefix('#').expect("local $ref");
                        assert!(doc.pointer(ptr).is_some(), "{at}: {r} does not resolve");
                    }
                    for (k, v) in m {
                        walk(v, doc, &format!("{at}/{k}"));
                    }
                }
                Value::Array(a) => a.iter().for_each(|v| walk(v, doc, at)),
                _ => {}
            }
        }
        walk(&doc, &doc, "");
    }

    #[test]
    fn the_document_is_served_at_both_paths_as_json() {
        for path in ["/openapi.json", "/.well-known/openapi.json"] {
            let r = site::reply("GET", path).unwrap();
            assert_eq!(r.status, 200, "{path}");
            assert_eq!(r.header("content-type"), Some("application/json"), "{path}");
            assert_eq!(
                r.header("cache-control"),
                Some("public, max-age=300"),
                "{path}"
            );
            assert_eq!(r.header("access-control-allow-origin"), Some("*"), "{path}");
            assert_eq!(r.body.as_deref(), Some(OPENAPI.as_bytes()), "{path}");
        }
    }

    #[test]
    fn every_page_links_the_document_as_its_service_description() {
        let html = String::from_utf8(site::reply("GET", "/").unwrap().body.unwrap()).unwrap();
        assert!(html.contains(
            r#"<link rel="service-desc" type="application/openapi+json" href="/openapi.json"/>"#
        ));
    }

    #[test]
    fn the_matcher_works() {
        assert!(matches("/register/{id}/", "/register/h3/"));
        assert!(!matches("/register/{id}/", "/register//"));
        assert!(!matches("/register/{id}/", "/register/h3/x/"));
        let t = vec!["/register/latest".to_owned(), "/register/".to_owned()];
        assert_eq!(covered(&t, "/register/latest/"), Some("/register/latest"));
        assert_eq!(covered(&t, "/register"), Some("/register/"));
        assert_eq!(covered(&t, "/nope"), None);
        assert_eq!(normalise("/cell/../../register/x.yaml"), "/register/x.yaml");
        assert_eq!(normalise("/a/./b/.."), "/a/");
        assert_eq!(
            links(r#"<a href="results.md#x"></a><img src="//cdn/x">"#, "/w/"),
            ["/w/results.md"]
        );
    }
}
