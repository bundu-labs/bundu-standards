//! Reads `register/register.yaml` once, at build time, and writes it as JSON for the Worker.
//!
//! The YAML parser stays out of the wasm: the Worker only parses the JSON this writes. The
//! register is checked here too (through the same `register.rs` the Worker uses), so a bad
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
    fs::write(out, json).unwrap();
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
