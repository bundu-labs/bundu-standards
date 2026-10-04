//! vocab.bundu.org — the Bundu vocabulary namespace.
//!
//! A vocabulary URI has one job: when something dereferences
//! `https://vocab.bundu.org/Family`, it must get back a useful answer. A
//! machine wants JSON-LD. A person wants a page they can read. This serves
//! both from the same URI by content negotiation, which is what makes the
//! namespace honest rather than decorative — plenty of vocabularies publish
//! URIs that 404, and consumers quietly stop trusting them.
//!
//! There is no database and no state. The vocabulary is compiled in, so the
//! deployed artefact is a single binary that cannot serve a term the source
//! does not define.

mod terms;

use std::net::SocketAddr;

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use terms::{Domain, Status, Term};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

/// The namespace root. Every term URI is this plus the term id.
const NAMESPACE: &str = "https://vocab.bundu.org/";

const JSON_LD: &str = "application/ld+json";

/// W3C vocabulary status, for `vs:term_status`.
const VS: &str = "http://www.w3.org/2003/06/sw-vocab-status/ns#";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(|| async { "ok" }))
        .route("/context.jsonld", get(context))
        .route("/:term", get(term))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("LISTEN_ADDR must be host:port");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("cannot bind");
    tracing::info!("bundu-vocab listening on {addr}");
    axum::serve(listener, app).await.expect("server failed");
}

/// Does the caller want HTML?
///
/// Browsers ask for `text/html` explicitly; RDF tooling asks for JSON-LD or
/// sends no preference at all. Defaulting to JSON-LD means `curl` gets data
/// rather than markup, which is the more useful default for a namespace.
fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("text/html"))
}

fn jsonld(value: Value) -> Response {
    (
        [
            (header::CONTENT_TYPE, JSON_LD),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            // A vocabulary is meant to be read by anyone, from anywhere.
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        ],
        value.to_string(),
    )
        .into_response()
}

fn html(body: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        body,
    )
        .into_response()
}

// ── JSON-LD ──────────────────────────────────────────────────────────────

fn term_jsonld(t: &Term) -> Value {
    let mut v = json!({
        "@context": {
            "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
            "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
            "schema": "https://schema.org/",
            "bundu": NAMESPACE,
            "vs": VS,
        },
        "@id": format!("bundu:{}", t.id),
        "@type": t.kind.rdf_type(),
        "rdfs:label": t.label,
        "rdfs:comment": t.comment,
        "rdfs:isDefinedBy": NAMESPACE,
        "bundu:rationale": t.rationale,
        "vs:term_status": t.status.vs_term_status(),
    });

    let obj = v.as_object_mut().expect("object");
    obj.insert("bundu:subjectArea".into(), json!(t.domain.label()));
    if let Some(parent) = t.sub_class_of {
        obj.insert("rdfs:subClassOf".into(), json!({ "@id": parent }));
    }
    if let Some(applies_to) = t.domain_includes {
        obj.insert("schema:domainIncludes".into(), json!({ "@id": applies_to }));
    }
    if let Some(range) = t.range {
        obj.insert("schema:rangeIncludes".into(), json!({ "@id": range }));
    }
    if let Some(values) = t.values {
        // Enumerated properties publish their permitted values, so an
        // implementer never has to guess the spelling of one.
        obj.insert(
            "schema:valueReference".into(),
            json!(values
                .iter()
                .map(
                    |(v, desc)| json!({ "@type": "rdfs:Literal", "value": v, "rdfs:comment": desc })
                )
                .collect::<Vec<_>>()),
        );
    }
    v
}

async fn context() -> Response {
    // The @context consumers embed to use these terms by short name.
    let mut map = serde_json::Map::new();
    map.insert("schema".into(), json!("https://schema.org/"));
    map.insert("bundu".into(), json!(NAMESPACE));
    for t in terms::TERMS {
        map.insert(t.id.into(), json!(format!("bundu:{}", t.id)));
    }
    jsonld(json!({ "@context": Value::Object(map) }))
}

// ── Handlers ─────────────────────────────────────────────────────────────

async fn index(headers: HeaderMap) -> Response {
    if !wants_html(&headers) {
        return jsonld(json!({
            "@context": {
                "bundu": NAMESPACE,
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "vs": VS,
            },
            "@id": NAMESPACE,
            "rdfs:label": "The Bundu vocabulary",
            "rdfs:comment": "RDF terms for concepts schema.org has no word for.",
            "termCount": terms::TERMS.len(),
            "terms": terms::TERMS.iter()
                .map(|t| json!({
                    "@id": format!("bundu:{}", t.id),
                    "@type": t.kind.rdf_type(),
                    "rdfs:label": t.label,
                    "vs:term_status": t.status.vs_term_status(),
                }))
                .collect::<Vec<_>>(),
        }));
    }

    // Grouped by subject area rather than by class/property: someone
    // arriving here is looking for a topic, not a JSON-LD construct.
    let sections = Domain::ALL
        .iter()
        .filter(|d| terms::in_domain(**d).next().is_some())
        .map(|d| {
            let items = terms::in_domain(*d)
                .map(|t| {
                    format!(
                        r#"<li><a href="/{id}"><code>{id}</code></a><span>{kind}{pending}</span><p>{comment}</p></li>"#,
                        id = t.id,
                        kind = t.kind.label().to_lowercase(),
                        pending = match t.status {
                            Status::Pending => format!(" · {}", t.status.label()),
                            Status::Stable => String::new(),
                        },
                        comment = esc(t.comment),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("<section>\n  <h2>{}</h2>\n  <ul class=\"terms\">{items}</ul>\n</section>", d.label())
        })
        .collect::<Vec<_>>()
        .join("\n");

    html(page(
        "The Bundu vocabulary",
        &format!(
            r#"<header>
  <p class="eyebrow">vocab.bundu.org</p>
  <h1>The Bundu vocabulary</h1>
  <p class="lede">RDF terms for concepts schema.org has no word for, published
  for every Bundu standard across every domain. Each term had to earn its
  place: if schema.org could express it, we use schema.org and add nothing.</p>
</header>
{sections}
<section>
  <h2>Using these terms</h2>
  <p>Fetch any term URI with <code>Accept: application/ld+json</code> for its
  machine-readable definition, or embed
  <a href="/context.jsonld"><code>/context.jsonld</code></a> to refer to terms
  by short name.</p>
  <p>Terms extend rather than replace schema.org. A family serialises as a
  <code>schema:Organization</code> carrying
  <code>additionalType: {ns}Family</code>, so consumers that have never heard of
  this vocabulary still read something valid.</p>
  <p>Term URIs are flat and permanent. Subject areas group this page only —
  they never appear in a URI, because a path segment encodes a taxonomy that
  will eventually be wrong, and by then the URI is already in someone's data.</p>
</section>"#,
            sections = sections,
            ns = NAMESPACE,
        ),
    ))
}

async fn term(Path(name): Path<String>, headers: HeaderMap) -> Response {
    let Some(t) = terms::find(&name) else {
        return not_found(&name, wants_html(&headers));
    };

    if !wants_html(&headers) {
        return jsonld(term_jsonld(t));
    }

    let mut rows = vec![
        ("Type".to_string(), t.kind.label().to_string()),
        ("Subject area".to_string(), t.domain.label().to_string()),
        ("Status".to_string(), status_cell(t.status)),
        (
            "URI".to_string(),
            format!("<code>{NAMESPACE}{}</code>", t.id),
        ),
    ];
    if let Some(p) = t.sub_class_of {
        rows.push(("Subclass of".into(), format!("<code>{p}</code>")));
    }
    if let Some(d) = t.domain_includes {
        rows.push(("Applies to".into(), format!("<code>{d}</code>")));
    }
    if let Some(r) = t.range {
        rows.push(("Value".into(), format!("<code>{r}</code>")));
    }
    let table = rows
        .iter()
        .map(|(k, v)| format!("<tr><th>{k}</th><td>{v}</td></tr>"))
        .collect::<Vec<_>>()
        .join("\n");

    let values_section = t.values.map_or_else(String::new, |values| {
        let rows = values
            .iter()
            .map(|(v, desc)| format!("<tr><th><code>{v}</code></th><td>{}</td></tr>", esc(desc)))
            .collect::<Vec<_>>()
            .join("\n");
        format!("<section>\n  <h2>Permitted values</h2>\n  <table>{rows}</table>\n</section>")
    });

    html(page(
        t.label,
        &format!(
            r#"<header>
  <p class="eyebrow"><a href="/">vocab.bundu.org</a> / {kind}</p>
  <h1><code class="term">{id}</code></h1>
  <p class="lede">{comment}</p>
</header>
<table>{table}</table>
{values_section}
<section>
  <h2>Why this term exists</h2>
  <p>{rationale}</p>
</section>
<section>
  <h2>Machine-readable</h2>
  <pre><code>curl -H 'Accept: application/ld+json' \
  {ns}{id}</code></pre>
</section>"#,
            kind = t.kind.label(),
            id = t.id,
            comment = esc(t.comment),
            rationale = esc(t.rationale),
            table = table,
            values_section = values_section,
            ns = NAMESPACE,
        ),
    ))
}

/// The status row on a term page. Pending is spelled out, because someone
/// about to embed the URI needs to know the definition may still move.
fn status_cell(status: Status) -> String {
    match status {
        Status::Stable => "Stable".to_string(),
        Status::Pending => "Pending: its standard is still being written, so the \
                            definition may be refined. The URI is permanent."
            .to_string(),
    }
}

fn not_found(name: &str, as_html: bool) -> Response {
    if as_html {
        return (
            StatusCode::NOT_FOUND,
            html(page(
                "Not a term",
                &format!(
                    r#"<header>
  <p class="eyebrow"><a href="/">vocab.bundu.org</a></p>
  <h1>Not a term</h1>
  <p class="lede">This vocabulary defines no term called
  <code>{}</code>. See <a href="/">the index</a> for what it does define.</p>
</header>"#,
                    esc(name)
                ),
            )),
        )
            .into_response();
    }
    (
        StatusCode::NOT_FOUND,
        jsonld(json!({
            "error": "unknown_term",
            "term": name,
            "namespace": NAMESPACE,
        })),
    )
        .into_response()
}

// ── Rendering ────────────────────────────────────────────────────────────

/// Escape text destined for HTML. Vocabulary content is compiled in and
/// therefore trusted, but the 404 path echoes a URL segment back, and
/// escaping everywhere is cheaper than remembering which path is which.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn page(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Bundu vocabulary</title>
<style>
:root {{
  --ground: #FAF9F5; --surface: #fff; --line: #E2DFD6;
  --text: #17161A; --muted: #6A6675; --accent: #5B21B6;
}}
@media (prefers-color-scheme: dark) {{
  :root {{
    --ground: #0B0B0D; --surface: #131317; --line: #2B2B34;
    --text: #F2F1EE; --muted: #9A98A6; --accent: #B388FF;
  }}
}}
* {{ box-sizing: border-box; }}
body {{
  background: var(--ground); color: var(--text);
  font: 16px/1.6 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  margin: 0; padding: 0 24px 96px;
}}
main {{ max-width: 680px; margin: 0 auto; }}
header {{ padding: 72px 0 8px; }}
h1 {{ font-size: 2.2rem; line-height: 1.1; letter-spacing: -.03em; margin: .2em 0; }}
h2 {{ font-size: 1.05rem; letter-spacing: -.01em; margin: 2.4rem 0 .6rem; }}
p {{ margin: .7em 0; }}
.eyebrow {{
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: .72rem; letter-spacing: .14em; text-transform: uppercase;
  color: var(--muted); margin: 0;
}}
.lede {{ font-size: 1.05rem; color: var(--muted); }}
a {{ color: var(--accent); text-underline-offset: 2px; }}
a:focus-visible {{ outline: 2px solid var(--accent); outline-offset: 3px; border-radius: 3px; }}
code {{
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: .88em; background: var(--surface);
  border: 1px solid var(--line); border-radius: 4px; padding: .08em .34em;
}}
code.term {{ font-size: .8em; background: none; border: none; padding: 0; }}
pre {{ overflow-x: auto; background: var(--surface); border: 1px solid var(--line);
      border-radius: 8px; padding: 1rem; }}
pre code {{ background: none; border: none; padding: 0; }}
table {{ border-collapse: collapse; width: 100%; margin: 1.6rem 0; }}
th, td {{ text-align: left; padding: .55rem .2rem; border-bottom: 1px solid var(--line);
         vertical-align: top; }}
th {{ width: 9rem; font-weight: 600; color: var(--muted); font-size: .88rem; }}
ul.terms {{ list-style: none; padding: 0; margin: 0; }}
ul.terms li {{ padding: 1rem 0; border-bottom: 1px solid var(--line); }}
ul.terms li span {{ color: var(--muted); font-size: .88rem; margin-left: .6rem; }}
ul.terms li p {{ margin: .35em 0 0; color: var(--muted); font-size: .93rem; }}
</style>
</head>
<body><main>
{body}
</main></body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn accept(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::ACCEPT, HeaderValue::from_str(value).unwrap());
        h
    }

    /// Browsers ask for HTML explicitly; RDF tooling asks for JSON-LD or
    /// nothing. Defaulting to JSON-LD is what makes `curl <term>` useful.
    #[test]
    fn content_negotiation_defaults_to_json_ld() {
        assert!(wants_html(&accept("text/html,application/xhtml+xml")));
        assert!(!wants_html(&accept("application/ld+json")));
        assert!(!wants_html(&accept("*/*")));
        assert!(!wants_html(&HeaderMap::new()));
    }

    #[test]
    fn every_term_serialises_with_its_contract() {
        for t in terms::TERMS {
            let v = term_jsonld(t);
            assert_eq!(v["@id"], format!("bundu:{}", t.id));
            assert_eq!(v["@type"], t.kind.rdf_type());
            assert_eq!(v["rdfs:isDefinedBy"], NAMESPACE);
            assert!(
                !v["bundu:rationale"].as_str().unwrap_or_default().is_empty(),
                "{}: rationale must survive into the published JSON-LD",
                t.id
            );
            assert_eq!(v["bundu:subjectArea"], t.domain.label());
        }
    }

    #[test]
    fn classes_publish_their_schema_org_parent() {
        let family = terms::find("Family").unwrap();
        let v = term_jsonld(family);
        assert_eq!(v["rdfs:subClassOf"]["@id"], "schema:Organization");
        // A family that did not serialise as an Organization would be
        // invisible to every consumer that has not heard of this vocabulary.
    }

    #[test]
    fn enumerated_properties_publish_their_values() {
        let v = term_jsonld(terms::find("recognitionBasis").unwrap());
        let published: Vec<&str> = v["schema:valueReference"]
            .as_array()
            .expect("values published")
            .iter()
            .map(|e| e["value"].as_str().unwrap())
            .collect();
        assert!(published.contains(&"customary"));
        assert!(published.contains(&"contested"));
        assert_eq!(published.len(), 6);
    }

    #[test]
    fn non_enumerated_properties_publish_no_values() {
        let v = term_jsonld(terms::find("verificationTier").unwrap());
        assert!(v.get("schema:valueReference").is_none());
    }

    #[test]
    fn escaping_protects_the_one_path_that_echoes_input() {
        // The 404 page reflects a URL segment back. Everything else is
        // compiled in, but escaping everywhere is cheaper than remembering
        // which path is which.
        let out = esc("<script>alert(1)</script>");
        assert!(!out.contains('<') && !out.contains('>'));
        assert!(out.contains("&lt;script&gt;"));
    }

    async fn body(r: Response) -> String {
        let bytes = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .expect("body");
        String::from_utf8(bytes.to_vec()).expect("utf-8")
    }

    fn content_type(r: &Response) -> String {
        r.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .to_string()
    }

    /// The contract every term URI makes, checked through the real handler
    /// for every term: it resolves, defaults to JSON-LD, the JSON-LD is valid
    /// and names the term, and a browser gets a page that renders it.
    #[tokio::test]
    async fn every_term_resolves_in_both_representations() {
        for t in terms::TERMS {
            // No Accept header: JSON-LD.
            let r = term(Path(t.id.to_string()), HeaderMap::new()).await;
            assert_eq!(r.status(), StatusCode::OK, "{}", t.id);
            assert_eq!(content_type(&r), JSON_LD, "{}", t.id);
            let v: Value = serde_json::from_str(&body(r).await)
                .unwrap_or_else(|e| panic!("{}: JSON-LD does not parse: {e}", t.id));
            assert_eq!(v["@id"], format!("bundu:{}", t.id));
            assert_eq!(v["vs:term_status"], t.status.vs_term_status());

            // Explicit JSON-LD.
            let r = term(Path(t.id.to_string()), accept(JSON_LD)).await;
            assert_eq!(r.status(), StatusCode::OK, "{}", t.id);
            assert_eq!(content_type(&r), JSON_LD, "{}", t.id);

            // A browser.
            let r = term(
                Path(t.id.to_string()),
                accept("text/html,application/xhtml+xml"),
            )
            .await;
            assert_eq!(r.status(), StatusCode::OK, "{}", t.id);
            assert!(content_type(&r).starts_with("text/html"), "{}", t.id);
            let page = body(r).await;
            assert!(page.starts_with("<!doctype html>"), "{}", t.id);
            assert!(
                page.contains(&format!(r#"<code class="term">{}</code>"#, t.id)),
                "{}: page does not name the term",
                t.id
            );
            assert!(
                page.contains(&esc(t.comment)),
                "{}: definition missing",
                t.id
            );
            assert!(
                page.contains(&esc(t.rationale)),
                "{}: rationale missing",
                t.id
            );
            assert!(
                page.contains(t.domain.label()),
                "{}: subject area missing",
                t.id
            );
            if t.status == Status::Pending {
                assert!(
                    page.contains("Pending"),
                    "{}: pending status not shown",
                    t.id
                );
            }
        }
    }

    /// Every prefixed name in a published definition must use a prefix its
    /// own @context declares, or a JSON-LD processor silently drops it.
    #[test]
    fn every_term_uses_only_declared_prefixes() {
        for t in terms::TERMS {
            let v = term_jsonld(t);
            let ctx = v["@context"].as_object().expect("context");
            let obj = v.as_object().unwrap();
            let mut names: Vec<String> = obj
                .keys()
                .filter(|k| !k.starts_with('@'))
                .cloned()
                .collect();
            for key in [
                "@type",
                "rdfs:subClassOf",
                "schema:domainIncludes",
                "schema:rangeIncludes",
            ] {
                match &v[key] {
                    Value::String(s) => names.push(s.clone()),
                    Value::Object(o) => names.push(o["@id"].as_str().unwrap().to_string()),
                    _ => {}
                }
            }
            for n in names {
                let (prefix, _) = n
                    .split_once(':')
                    .unwrap_or_else(|| panic!("{}: {n} is not a prefixed name", t.id));
                assert!(
                    ctx.contains_key(prefix),
                    "{}: prefix {prefix} undeclared",
                    t.id
                );
            }
        }
    }

    #[tokio::test]
    async fn context_and_index_list_every_term() {
        let ctx: Value = serde_json::from_str(&body(context().await).await).unwrap();
        let idx_json: Value =
            serde_json::from_str(&body(index(HeaderMap::new()).await).await).unwrap();
        let idx_html = body(index(accept("text/html")).await).await;
        assert_eq!(idx_json["termCount"], terms::TERMS.len());
        for t in terms::TERMS {
            assert_eq!(ctx["@context"][t.id], format!("bundu:{}", t.id));
            assert!(
                idx_html.contains(&format!(r#"<a href="/{0}"><code>{0}</code></a>"#, t.id)),
                "{} missing from the index page",
                t.id
            );
        }
        for d in Domain::ALL {
            if terms::in_domain(*d).next().is_some() {
                assert!(
                    idx_html.contains(d.label()),
                    "{} section missing",
                    d.label()
                );
            }
        }
    }

    #[tokio::test]
    async fn unknown_terms_are_404_in_both_representations() {
        let r = term(Path("NotATerm".into()), HeaderMap::new()).await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        assert_eq!(content_type(&r), JSON_LD);
        let r = term(Path("aiguardrail".into()), accept("text/html")).await;
        assert_eq!(
            r.status(),
            StatusCode::NOT_FOUND,
            "lookup is case-sensitive"
        );
        assert!(content_type(&r).starts_with("text/html"));
    }

    #[test]
    fn subclass_of_a_local_class_publishes_its_bundu_parent() {
        let v = term_jsonld(terms::find("PlatformGuardrail").unwrap());
        assert_eq!(v["rdfs:subClassOf"]["@id"], "bundu:AIGuardrail");
        assert_eq!(v["vs:term_status"], "testing");
    }

    #[test]
    fn namespace_has_a_trailing_slash() {
        // Term URIs are built by concatenation; without it every published
        // URI would be wrong.
        assert!(NAMESPACE.ends_with('/'));
    }
}
