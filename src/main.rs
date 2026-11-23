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
use terms::{Domain, Kind, Literal, Status, Term};
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

    let app = app();

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

/// The routes. axum 0.8 writes a path parameter as `{term}`; the old `:term` form panics
/// when the router is built, which only a test that builds this router catches.
fn app() -> Router {
    Router::new()
        .route("/", get(index))
        .route(
            "/healthz",
            get(|| async {
                (
                    [
                        (axum::http::header::CONTENT_TYPE, "application/json"),
                        (axum::http::header::CACHE_CONTROL, "no-store"),
                    ],
                    r#"{"status":"ok"}"#,
                )
            }),
        )
        .route("/context.jsonld", get(context))
        .route(bundu_site::CSS_PATH, get(site_css))
        .route("/favicon-32x32.png", get(|| icon(FAVICON_32)))
        .route("/favicon-16x16.png", get(|| icon(FAVICON_16)))
        .route("/favicon-32x32-dark.png", get(|| icon(FAVICON_32_DARK)))
        .route("/favicon-16x16-dark.png", get(|| icon(FAVICON_16_DARK)))
        .route("/apple-touch-icon.png", get(|| icon(APPLE_TOUCH_ICON)))
        .route("/copy.js", get(copy_js))
        .route("/{term}", get(term))
        .layer(axum::middleware::map_response(security_headers))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
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
    if let Some(i) = t.individual {
        // An individual's @type is what it is an instance of, not
        // rdfs:Resource, so consumers read it as a DefinedTerm.
        obj.insert("@type".into(), json!(i.types));
        if let Some(code) = i.term_code {
            obj.insert("schema:termCode".into(), json!(code));
        }
        if let Some(set) = i.in_set {
            obj.insert("schema:inDefinedTermSet".into(), json!({ "@id": set }));
        }
        for (prop, value) in i.facts {
            obj.insert((*prop).into(), literal(*value));
        }
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

fn literal(value: Literal) -> Value {
    match value {
        Literal::Text(s) => json!(s),
        Literal::Boolean(b) => json!(b),
    }
}

/// The JSON-LD `@type` of a term: the RDF kind, or for an individual, what
/// it is an instance of.
fn jsonld_type(t: &Term) -> Value {
    match (t.kind, t.individual) {
        (Kind::Individual, Some(i)) => json!(i.types),
        _ => json!(t.kind.rdf_type()),
    }
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

// ── Brand ─────────────────────────────────────────────────────────────────

// The official Bundu mark (copper), as shipped by www.bundu.org: a
// light-surface and a dark-surface pair, picked by the browser's colour
// scheme, never redrawn. The same files are in workers/standards-api/public; a
// test keeps the two copies identical.
const FAVICON_32: &[u8] = include_bytes!("brand/favicon-32x32.png");
const FAVICON_16: &[u8] = include_bytes!("brand/favicon-16x16.png");
const FAVICON_32_DARK: &[u8] = include_bytes!("brand/favicon-32x32-dark.png");
const FAVICON_16_DARK: &[u8] = include_bytes!("brand/favicon-16x16-dark.png");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("brand/apple-touch-icon.png");

async fn icon(bytes: &'static [u8]) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        bytes,
    )
        .into_response()
}

// ── Handlers ─────────────────────────────────────────────────────────────

/// The script behind "Copy this page as": the same file standards.bundu.org serves from
/// workers/standards-api/public/copy.js. A test keeps the two identical.
const COPY_JS: &str = include_str!("copy.js");

async fn copy_js() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        COPY_JS,
    )
        .into_response()
}

/// The vocabulary as JSON-LD: what `/` answers to RDF tooling, and what its page copies.
fn index_jsonld() -> Value {
    json!({
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
                    "@type": jsonld_type(t),
                    "rdfs:label": t.label,
                    "vs:term_status": t.status.vs_term_status(),
                }))
                .collect::<Vec<_>>(),
    })
}

async fn index(headers: HeaderMap) -> Response {
    if !wants_html(&headers) {
        return jsonld(index_jsonld());
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
                        r#"<li><a href="/{id}"><code>{id}</code></a> <span>{kind}{pending}</span><p>{comment}</p></li>"#,
                        id = t.id,
                        kind = t.kind.label().to_lowercase(),
                        pending = match t.status {
                            Status::Pending => format!(" · {}", t.status.label()),
                            Status::Stable => String::new(),
                        },
                        comment = prose(t.comment),
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
        Some("/"),
        Some(index_jsonld()),
        &format!(
            r#"<header>
  <h1>The Bundu vocabulary</h1>
  <p class="lede">RDF terms for concepts schema.org has no word for, published
  for every Bundu standard across every domain. Each term had to earn its
  place: if schema.org can express it, we use schema.org and add nothing.</p>
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
    if let Some(i) = t.individual {
        rows.push((
            "Instance of".into(),
            i.types
                .iter()
                .map(|ty| format!("<code>{ty}</code>"))
                .collect::<Vec<_>>()
                .join(" "),
        ));
        if let Some(code) = i.term_code {
            rows.push(("Code".into(), format!("<code>{}</code>", esc(code))));
        }
        if let Some(set) = i.in_set {
            rows.push(("In set".into(), link(set)));
        }
        for (prop, value) in i.facts {
            let shown = match value {
                Literal::Text(s) => format!("<code>{}</code>", esc(s)),
                Literal::Boolean(b) => format!("<code>{b}</code>"),
            };
            rows.push((link(prop), shown));
        }
    }
    let table = rows
        .iter()
        .map(|(k, v)| format!(r#"<tr><th scope="row">{k}</th><td>{v}</td></tr>"#))
        .collect::<Vec<_>>()
        .join("\n");

    let values_section = t.values.map_or_else(String::new, |values| {
        let rows = values
            .iter()
            .map(|(v, desc)| format!(r#"<tr><th scope="row"><code>{v}</code></th><td>{}</td></tr>"#, prose(desc)))
            .collect::<Vec<_>>()
            .join("\n");
        format!("<section>\n  <h2>Permitted values</h2>\n  <table><caption class=\"sr-only\">Permitted values</caption>{rows}</table>\n</section>")
    });

    html(page(
        t.label,
        Some(&format!("/{}", t.id)),
        Some(term_jsonld(t)),
        &format!(
            r#"<header>
  <p class="eyebrow">{kind}</p>
  <h1><code class="term">{id}</code></h1>
  <p class="lede">{comment}</p>
</header>
<table><caption class="sr-only">About this term</caption>{table}</table>
{values_section}
<section>
  <h2>Why this term exists</h2>
  <p>{rationale}</p>
</section>
<section>
  <h2>Machine-readable</h2>
  <pre tabindex="0" role="region" aria-label="Example request"><code>curl -H 'Accept: application/ld+json' \
  {ns}{id}</code></pre>
</section>"#,
            kind = t.kind.label(),
            id = t.id,
            comment = prose(t.comment),
            rationale = prose(t.rationale),
            table = table,
            values_section = values_section,
            ns = NAMESPACE,
        ),
    ))
}

/// A prefixed name as HTML: a link when it is one of ours.
fn link(name: &str) -> String {
    match name.strip_prefix("bundu:") {
        Some(local) => format!(r#"<a href="/{local}"><code>{name}</code></a>"#),
        None => format!("<code>{}</code>", esc(name)),
    }
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
                None,
                None,
                &format!(
                    r#"<header>
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

/// Text as HTML, with `code` in backticks shown as code, as Markdown does. An unpaired
/// backtick stays as text.
fn prose(s: &str) -> String {
    let parts: Vec<&str> = s.split('`').collect();
    if parts.len() % 2 == 0 {
        return esc(s);
    }
    parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i % 2 == 1 {
                format!("<code>{}</code>", esc(p))
            } else {
                esc(p)
            }
        })
        .collect()
}

/// A page, in the shell that standards.bundu.org shares (`bundu-site`). `path` gives the
/// canonical link; `data` is what "Copy this page as" offers as JSON-LD (a page without it
/// offers Markdown only).
fn page(title: &str, path: Option<&str>, data: Option<Value>, body: &str) -> String {
    let site = bundu_site::Page {
        path: path.unwrap_or("/").to_owned(),
        canonical: path.is_some(),
        origin: NAMESPACE.trim_end_matches('/').to_owned(),
        // The home page is the site, so its title is the site name; every other page is
        // "<page> — <site>", with no word in both halves.
        title: if path == Some("/") {
            "Bundu vocabulary".to_owned()
        } else {
            format!("{title} — Bundu vocabulary")
        },
        description: "The Bundu vocabulary: RDF terms for concepts schema.org has no word for."
            .into(),
        brand: "Bundu vocabulary".into(),
        links: vec![
            bundu_site::NavLink::new("All terms", "/"),
            bundu_site::NavLink::new("JSON-LD context", "/context.jsonld"),
            bundu_site::NavLink::new(
                "Standards register",
                "https://standards.bundu.org/register/",
            ),
            bundu_site::NavLink::new(
                "Accessibility",
                "https://standards.bundu.org/accessibility/",
            ),
        ],
        json_label: "JSON-LD".into(),
        json_ld: None,
        // `</script>` cannot appear inside the JSON: every `<` is written as \u003c.
        page_json: data.map(|v| {
            (
                "application/ld+json".to_owned(),
                v.to_string().replace('<', "\\u003c"),
            )
        }),
    };
    let footer = r##"<p>Every page works with a keyboard alone: <kbd>Tab</kbd> moves forward, <kbd>Shift</kbd>+<kbd>Tab</kbd> moves back, <kbd>Enter</kbd> follows a link. Read the <a href="https://standards.bundu.org/accessibility/">accessibility statement</a> or <a href="https://github.com/bundu-labs/bundu-standards/issues/new?title=Accessibility%3A%20">report a barrier</a>.</p>"##;
    bundu_site::render_html(site, "", footer, body)
}

/// The shared stylesheet, the one standards.bundu.org serves.
async fn site_css() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
            (header::ETAG, bundu_site::CSS_ETAG),
        ],
        bundu_site::CSS,
    )
        .into_response()
}

/// The security headers and content security policy standards.bundu.org sends: nothing inline,
/// scripts and styles from this origin only.
async fn security_headers(mut res: Response) -> Response {
    let h = res.headers_mut();
    for (k, v) in [
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("x-frame-options", "DENY"),
        ("strict-transport-security", "max-age=31536000"),
        ("permissions-policy", "geolocation=(), camera=(), microphone=()"),
        (
            "content-security-policy",
            "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        ),
    ] {
        h.insert(
            header::HeaderName::from_static(k),
            header::HeaderValue::from_static(v),
        );
    }
    res
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
    /// The Bundu mark ships twice: compiled into this binary and in the
    /// standards site's public/ folder. Both must be the same files.
    #[test]
    fn brand_icons_match_the_standards_site() {
        let site =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("workers/standards-api/public");
        for (name, bytes) in [
            ("favicon-32x32.png", FAVICON_32),
            ("favicon-16x16.png", FAVICON_16),
            ("favicon-32x32-dark.png", FAVICON_32_DARK),
            ("favicon-16x16-dark.png", FAVICON_16_DARK),
            ("apple-touch-icon.png", APPLE_TOUCH_ICON),
        ] {
            assert!(bytes.starts_with(b"\x89PNG"), "{name} is not a PNG");
            assert_eq!(
                std::fs::read(site.join(name)).unwrap(),
                bytes,
                "{name} differs from the standards site"
            );
        }
    }

    /// Keyboard and screen-reader aids every page must keep.
    #[test]
    fn every_page_has_a_skip_link_landmarks_and_keyboard_help() {
        let html = page("t", None, None, "<p>b</p>");
        let body = html.find("<body>").unwrap();
        let skip = html
            .find(r##"<a class="skip" href="#main">Skip to main content</a>"##)
            .unwrap();
        assert!(
            skip > body && skip < html.find("<header").unwrap(),
            "skip link must come first"
        );
        assert!(html.contains(r#"<nav aria-label="Main">"#));
        assert!(html.contains(r#"<main id="main" tabindex="-1">"#));
        assert!(html.contains("<footer") && html.contains("<kbd>Tab</kbd>"));
        assert!(html.contains("report a barrier"));
        assert!(html.contains(r#"<html lang="en">"#));
        for img in html
            .match_indices("<img ")
            .map(|(i, _)| &html[i..i + html[i..].find('>').unwrap()])
        {
            assert!(
                img.contains(r#"alt="""#),
                "decorative image without alt=\"\": {img}"
            );
        }
    }

    #[tokio::test]
    async fn term_tables_have_captions_and_row_headers() {
        for t in terms::TERMS {
            let html = body(term(Path(t.id.to_string()), accept("text/html")).await).await;
            for (i, _) in html.match_indices("<table>") {
                assert!(
                    html[i..].starts_with(r#"<table><caption class="sr-only">"#),
                    "{}: table without a caption",
                    t.id
                );
            }
            assert!(
                !html.contains("<th>"),
                "{}: a header cell without scope",
                t.id
            );
        }
    }

    /// The stylesheet is the one standards.bundu.org serves, and it keeps the Bundu floor:
    /// APCA and WCAG contrast from its own tokens in both themes, focus, hit regions, spacing
    /// floors, reduced motion (see `bundu-site`'s `css_audit`).
    #[test]
    fn the_stylesheet_keeps_the_bundu_floor() {
        assert_eq!(bundu_site::css_audit::audit_shipped(), Vec::<String>::new());
    }

    /// Every page the vocabulary renders passes the shared accessibility audit: the index, every
    /// term (those with permitted values too) and the not-found page.
    #[tokio::test]
    async fn every_page_passes_the_shared_accessibility_audit() {
        async fn page_html(path: &str) -> String {
            use tower::ServiceExt;
            let req = axum::http::Request::builder()
                .uri(path)
                .header(header::ACCEPT, "text/html")
                .body(axum::body::Body::empty())
                .unwrap();
            body(app().oneshot(req).await.unwrap()).await
        }
        let mut pages = vec![("/".to_owned(), page_html("/").await)];
        for t in terms::TERMS {
            let p = format!("/{}", t.id);
            let html = page_html(&p).await;
            pages.push((p, html));
        }
        pages.push(("/notATerm".into(), page_html("/notATerm").await));
        for (p, html) in pages {
            let problems = bundu_site::audit::audit(&html);
            assert!(problems.is_empty(), "{p}: {problems:#?}");
        }
    }

    #[test]
    fn every_page_links_a_light_and_a_dark_icon() {
        let html = page("t", None, None, "<p>b</p>");
        assert!(html.contains(r#"href="/favicon-32x32.png" media="(prefers-color-scheme: light)""#));
        assert!(
            html.contains(r#"href="/favicon-32x32-dark.png" media="(prefers-color-scheme: dark)""#)
        );
    }

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
            assert_eq!(v["@type"], jsonld_type(t));
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
                "schema:inDefinedTermSet",
            ] {
                match &v[key] {
                    Value::String(s) => names.push(s.clone()),
                    Value::Object(o) => names.push(o["@id"].as_str().unwrap().to_string()),
                    Value::Array(a) => {
                        names.extend(a.iter().map(|x| x.as_str().unwrap().to_string()))
                    }
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
    async fn icons_are_served_as_png() {
        let res = icon(FAVICON_32_DARK).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[header::CONTENT_TYPE], "image/png");
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
        assert_eq!(v["vs:term_status"], "stable");
    }

    /// A BAGS rule is published as a DefinedTerm in the BAGS set, keeping
    /// its code, so `vocab.bundu.org/BAGS-01` is citable as that rule.
    #[tokio::test]
    async fn bags_rules_publish_as_coded_terms_in_their_set() {
        let v = term_jsonld(terms::find("BAGS-08").unwrap());
        assert_eq!(
            v["@type"],
            json!(["schema:DefinedTerm", "bundu:PlatformGuardrail"])
        );
        assert_eq!(v["schema:termCode"], "BAGS-08");
        assert_eq!(v["schema:inDefinedTermSet"]["@id"], "bundu:BAGS");
        assert_eq!(v["bundu:guardrailCategory"], "prompt_injection");
        assert_eq!(v["bundu:guardrailSeverity"], "critical");
        assert_eq!(v["bundu:isCoreGuardrail"], true);
        assert_eq!(v["bundu:standardVersion"], "1.0.0");

        let set = term_jsonld(terms::find("BAGS").unwrap());
        assert_eq!(set["@type"], json!(["schema:DefinedTermSet"]));

        let page = body(term(Path("BAGS-08".into()), accept("text/html")).await).await;
        assert!(page.contains(r#"<th scope="row">Code</th><td><code>BAGS-08</code></td>"#));
        assert!(page.contains(r#"<a href="/BAGS"><code>bundu:BAGS</code></a>"#));
        assert!(page.contains("<code>prompt_injection</code>"));
    }

    #[test]
    fn namespace_has_a_trailing_slash() {
        // Term URIs are built by concatenation; without it every published
        // URI would be wrong.
        assert!(NAMESPACE.ends_with('/'));
    }

    #[tokio::test]
    async fn every_page_offers_to_copy_itself_as_markdown_or_json_ld() {
        let html = body(index(accept("text/html")).await).await;
        assert!(html.contains(r#"<div class="copy" data-copy="" data-md-skip="" hidden="hidden">"#));
        assert!(html.contains(
            r#"<option value="markdown">Markdown</option><option value="json">JSON-LD</option>"#
        ));
        assert!(!html.contains(r#"value="html""#), "Markdown, not HTML");
        assert!(html.contains(r#"<script src="/copy.js" defer="defer">"#));
        assert!(html.contains(r#"<link rel="canonical" href="https://vocab.bundu.org/"/>"#));
        let t = &terms::TERMS[0];
        let term_page = body(term(Path(t.id.to_string()), accept("text/html")).await).await;
        let start = term_page
            .find(r#"<script type="application/ld+json" id="page-json">"#)
            .unwrap();
        let data = &term_page[start..start + term_page[start..].find("</script>").unwrap()];
        assert!(data.contains(&format!(r#""@id":"bundu:{}""#, t.id)));
    }

    #[test]
    fn the_copy_script_is_the_same_file_on_both_sites() {
        assert_eq!(
            COPY_JS,
            include_str!("../workers/standards-api/public/copy.js"),
            "src/copy.js and workers/standards-api/public/copy.js must stay identical"
        );
    }

    #[test]
    fn backticks_in_definitions_render_as_code() {
        assert_eq!(
            prose("one boolean per channel (today `email`)"),
            "one boolean per channel (today <code>email</code>)"
        );
        assert_eq!(prose("a ` b <c>"), "a ` b &lt;c&gt;");
        for t in terms::TERMS.iter() {
            assert!(
                !prose(t.comment).contains('`'),
                "{}: unpaired backtick",
                t.id
            );
        }
    }

    #[tokio::test]
    async fn term_pages_reflow_on_a_narrow_screen() {
        let html =
            body(term(Path("notificationPreference".into()), accept("text/html")).await).await;
        assert!(
            bundu_site::css_audit::PAGE_SOURCE.contains("overflow-wrap: anywhere"),
            "long term names and URIs wrap (the shared stylesheet)"
        );
        assert!(
            html.contains(r#"<pre tabindex="0" role="region" aria-label="Example request">"#),
            "the example block takes the focus, so it can scroll with the keyboard"
        );
    }

    /// The real router, as `main` serves it: building it would panic on a path syntax the
    /// axum version rejects, and every route must answer.
    #[tokio::test]
    async fn the_router_builds_and_every_route_answers() {
        use tower::ServiceExt;
        for (path, status) in [
            ("/", StatusCode::OK),
            ("/healthz", StatusCode::OK),
            ("/context.jsonld", StatusCode::OK),
            ("/copy.js", StatusCode::OK),
            ("/favicon-32x32.png", StatusCode::OK),
            ("/Family", StatusCode::OK),
            ("/notATerm", StatusCode::NOT_FOUND),
        ] {
            let req = axum::http::Request::builder()
                .uri(path)
                .body(axum::body::Body::empty())
                .unwrap();
            let res = app().oneshot(req).await.unwrap();
            assert_eq!(res.status(), status, "{path}");
        }
    }
}
