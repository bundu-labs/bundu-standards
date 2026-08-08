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
use terms::{Kind, Term};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

/// The namespace root. Every term URI is this plus the term id.
const NAMESPACE: &str = "https://vocab.bundu.org/";

const JSON_LD: &str = "application/ld+json";

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
        },
        "@id": format!("bundu:{}", t.id),
        "@type": t.kind.rdf_type(),
        "rdfs:label": t.label,
        "rdfs:comment": t.comment,
        "rdfs:isDefinedBy": NAMESPACE,
        "bundu:rationale": t.rationale,
    });

    let obj = v.as_object_mut().expect("object");
    if let Some(parent) = t.sub_class_of {
        obj.insert("rdfs:subClassOf".into(), json!({ "@id": parent }));
    }
    if let Some(domain) = t.domain {
        obj.insert("schema:domainIncludes".into(), json!({ "@id": domain }));
    }
    if let Some(range) = t.range {
        obj.insert("schema:rangeIncludes".into(), json!({ "@id": range }));
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
            "@context": { "bundu": NAMESPACE, "rdfs": "http://www.w3.org/2000/01/rdf-schema#" },
            "@id": NAMESPACE,
            "rdfs:label": "The Bundu vocabulary",
            "rdfs:comment": "RDF terms for concepts schema.org has no word for.",
            "termCount": terms::TERMS.len(),
            "terms": terms::TERMS.iter()
                .map(|t| json!({
                    "@id": format!("bundu:{}", t.id),
                    "@type": t.kind.rdf_type(),
                    "rdfs:label": t.label,
                }))
                .collect::<Vec<_>>(),
        }));
    }

    let section = |kind: Kind| -> String {
        terms::of_kind(kind)
            .map(|t| {
                format!(
                    r#"<li><a href="/{id}"><code>{id}</code></a><span>{label}</span><p>{comment}</p></li>"#,
                    id = t.id,
                    label = t.label,
                    comment = esc(t.comment),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    html(page(
        "The Bundu vocabulary",
        &format!(
            r#"<header>
  <p class="eyebrow">vocab.bundu.org</p>
  <h1>The Bundu vocabulary</h1>
  <p class="lede">RDF terms for concepts schema.org has no word for. Every term
  here had to earn its place: if schema.org could express it, we use schema.org
  and add nothing.</p>
</header>
<section>
  <h2>Classes</h2>
  <ul class="terms">{classes}</ul>
</section>
<section>
  <h2>Properties</h2>
  <ul class="terms">{properties}</ul>
</section>
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
</section>"#,
            classes = section(Kind::Class),
            properties = section(Kind::Property),
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
        ("URI".to_string(), format!("<code>{NAMESPACE}{}</code>", t.id)),
    ];
    if let Some(p) = t.sub_class_of {
        rows.push(("Subclass of".into(), format!("<code>{p}</code>")));
    }
    if let Some(d) = t.domain {
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

    html(page(
        t.label,
        &format!(
            r#"<header>
  <p class="eyebrow"><a href="/">vocab.bundu.org</a> / {kind}</p>
  <h1><code class="term">{id}</code></h1>
  <p class="lede">{comment}</p>
</header>
<table>{table}</table>
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
            ns = NAMESPACE,
        ),
    ))
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
