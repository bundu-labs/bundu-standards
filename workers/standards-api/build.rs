//! Reads `register/register.yaml` once, at build time, and writes it as JSON for the server.
//!
//! The YAML parser stays out of the binary: the server only parses the JSON this writes. The
//! register is checked here too (through the same `register.rs` the server uses), so a bad
//! register fails the build instead of the first request.

use std::{env, fs, path::Path};

#[path = "src/register.rs"]
#[allow(dead_code)]
mod register;

const SOURCE: &str = "../../register/register.yaml";

fn main() {
    println!("cargo:rerun-if-changed={SOURCE}");
    let text = fs::read_to_string(SOURCE).expect("read register/register.yaml");
    let doc: serde_yaml::Value = serde_yaml::from_str(&text).expect("register.yaml is YAML");

    // `domains` is a mapping whose order is the order of the sections on the page. A JSON
    // object would lose it, so it goes over as a list of [key, title] pairs.
    let domains: Vec<(String, String)> = doc
        .get("domains")
        .and_then(|d| d.as_mapping())
        .expect("register.yaml: expected `domains`")
        .iter()
        .map(|(k, v)| (scalar(k), scalar(v)))
        .collect();
    let standards = doc
        .get("standards")
        .and_then(|s| s.as_sequence())
        .expect("register.yaml: expected `standards`")
        .iter()
        .map(to_json)
        .collect::<Vec<_>>();

    let json = serde_json::json!({ "domains": domains, "standards": standards }).to_string();
    if let Err(e) = register::Register::parse(&json) {
        panic!("{e}");
    }
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("register.json");
    fs::write(&out, &json).unwrap();
    documents(&json);
    assets();
}

/// Our own standards, published beside the register: every entry whose `url` is a page on
/// standards.bundu.org gets that page, built from `standards/<slug>/`. The README becomes the
/// page (rendered here, so no Markdown parser ships) and the data files are served
/// as they are. A register URL with no folder behind it fails the build, so the register can
/// never link to a page that does not exist.
fn documents(register_json: &str) {
    println!("cargo:rerun-if-changed={STANDARDS}");
    let reg: serde_json::Value = serde_json::from_str(register_json).unwrap();
    let mut docs = serde_json::Map::new();
    for s in reg["standards"].as_array().unwrap() {
        let url = s["url"].as_str().unwrap_or_default();
        let Some(slug) = url
            .strip_prefix("https://standards.bundu.org/")
            .and_then(|r| r.strip_suffix('/'))
        else {
            continue;
        };
        if slug.starts_with("register") || slug.contains('/') {
            continue;
        }
        let dir = Path::new(STANDARDS).join(slug);
        let readme = fs::read_to_string(dir.join("README.md")).unwrap_or_else(|_| {
            panic!(
                "register {}: url {url} has no standards/{slug}/README.md behind it",
                s["id"].as_str().unwrap_or("?")
            )
        });
        let mut files = serde_json::Map::new();
        collect(&dir, &dir, &mut files);
        docs.insert(
            slug.to_owned(),
            serde_json::json!({ "id": s["id"], "html": markdown(&readme), "files": files }),
        );
    }
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("documents.json");
    fs::write(out, serde_json::Value::Object(docs).to_string()).unwrap();
}

const STANDARDS: &str = "../../standards";

/// The published files of a standard's folder: documents and data, not its tools or tests.
fn collect(root: &Path, dir: &Path, out: &mut serde_json::Map<String, serde_json::Value>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name.starts_with("__") {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, out);
        } else if matches!(
            path.extension().and_then(|x| x.to_str()),
            Some("md" | "json" | "jsonld" | "ndjson")
        ) {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel, fs::read_to_string(&path).unwrap().into());
        }
    }
}

fn markdown(text: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(text, Options::ENABLE_TABLES));
    // A code block can scroll sideways, so it has to take the keyboard focus (WCAG 2.1.1).
    // A table's header cells name their column (WCAG 1.3.1). pulldown-cmark writes a plain
    // `<th>` for an unaligned column and a `style` attribute for an aligned one, which the
    // content security policy forbids; the tables in our READMEs are all unaligned.
    assert!(
        !out.contains("<th style"),
        "an aligned Markdown table column: use none"
    );
    out.replace("<pre>", "<pre tabindex=\"0\">")
        .replace("<th>", "<th scope=\"col\">")
}

/// YAML to JSON, with every scalar as a string: the pages print values, they never count
/// with them, and `1.0` must stay `1.0`.
fn to_json(v: &serde_yaml::Value) -> serde_json::Value {
    use serde_yaml::Value as Y;
    match v {
        Y::Mapping(m) => {
            serde_json::Value::Object(m.iter().map(|(k, v)| (scalar(k), to_json(v))).collect())
        }
        Y::Sequence(s) => serde_json::Value::Array(s.iter().map(to_json).collect()),
        Y::Null => serde_json::Value::Null,
        Y::Bool(b) => serde_json::Value::Bool(*b),
        other => serde_json::Value::String(scalar(other)),
    }
}

fn scalar(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        other => panic!("register.yaml: expected a scalar, found {other:?}"),
    }
}

const PUBLIC: &str = "public";

/// Lists `public/` for `assets.rs`: the URL path and the bytes of every file but `_headers`
/// (which `assets.rs` reads as rules, not as a file to serve). A new file there is served
/// with no code change.
fn assets() {
    println!("cargo:rerun-if-changed={PUBLIC}");
    let root = fs::canonicalize(PUBLIC).expect("public/ exists");
    let mut entries: Vec<_> = fs::read_dir(&root).unwrap().flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    let mut out = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name.starts_with('_') || !e.path().is_file() {
            continue;
        }
        println!("cargo:rerun-if-changed={PUBLIC}/{name}");
        out.push_str(&format!(
            "    ({:?}, include_bytes!({:?})),\n",
            format!("/{name}"),
            e.path().to_string_lossy()
        ));
    }
    out.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        out,
    )
    .unwrap();
}
