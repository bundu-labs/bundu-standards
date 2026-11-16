//! The register, for the pages: the Rust port of `workers/standards/src/lib/register.js`.
//!
//! Pure, with no platform dependency. `build.rs` turns `register/register.yaml` into JSON;
//! [`Register::parse`] reads that JSON and checks it the way `parseRegister` did.

use serde::{Deserialize, Serialize};

pub const SITE: &str = "https://standards.bundu.org";

#[derive(Debug, Clone, Deserialize)]
pub struct Standard {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub steward: Option<String>,
    pub commitment: String,
    pub current: String,
    pub governs: String,
    pub status: String,
    pub decided: String,
    pub url: String,
    pub docs: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub key: String,
    pub title: String,
    pub standards: Vec<Standard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub total: usize,
    pub adopted: usize,
    pub proposed: usize,
}

#[derive(Debug, Clone)]
pub struct Register {
    pub domains: Vec<Domain>,
    pub standards: Vec<Standard>,
    pub counts: Counts,
}

#[derive(Deserialize)]
struct Raw {
    domains: Vec<(String, String)>,
    standards: Vec<Standard>,
}

impl Register {
    /// Read the JSON `build.rs` writes, and check it: unique ids, known domains, and a
    /// status of `adopted` or `proposed`.
    pub fn parse(json: &str) -> Result<Self, String> {
        let raw: Raw = serde_json::from_str(json).map_err(|e| format!("register.yaml: {e}"))?;
        let mut seen = std::collections::HashSet::new();
        for s in &raw.standards {
            if s.id.is_empty() || !seen.insert(s.id.as_str()) {
                return Err(format!("register.yaml: missing or duplicate id {}", s.id));
            }
            if !raw.domains.iter().any(|(k, _)| *k == s.domain) {
                return Err(format!(
                    "register.yaml: {}: unknown domain {}",
                    s.id, s.domain
                ));
            }
            if s.status != "adopted" && s.status != "proposed" {
                return Err(format!("register.yaml: {}: bad status", s.id));
            }
        }
        let domains = raw
            .domains
            .iter()
            .map(|(key, title)| Domain {
                key: key.clone(),
                title: title.clone(),
                standards: raw
                    .standards
                    .iter()
                    .filter(|s| s.domain == *key)
                    .cloned()
                    .collect(),
            })
            .filter(|d| !d.standards.is_empty())
            .collect();
        let count = |status: &str| raw.standards.iter().filter(|s| s.status == status).count();
        let counts = Counts {
            total: raw.standards.len(),
            adopted: count("adopted"),
            proposed: count("proposed"),
        };
        Ok(Self {
            domains,
            standards: raw.standards,
            counts,
        })
    }

    pub fn get(&self, id: &str) -> Option<&Standard> {
        self.standards.iter().find(|s| s.id == id)
    }
}

/// Collapse runs of whitespace, as `governs.replace(/\s+/g, " ").trim()` did.
pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Serialize)]
pub struct DefinedTerm<'a> {
    #[serde(rename = "@type")]
    pub kind: &'static str,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "termCode")]
    pub term_code: &'a str,
    pub name: &'a str,
    pub description: String,
    pub url: &'a str,
    #[serde(rename = "inDefinedTermSet")]
    pub in_defined_term_set: String,
}

/// schema.org DefinedTerm for one standard.
pub fn defined_term<'a>(s: &'a Standard, origin: &str) -> DefinedTerm<'a> {
    DefinedTerm {
        kind: "DefinedTerm",
        id: format!("{origin}/register/{}/", s.id),
        term_code: &s.id,
        name: &s.name,
        description: one_line(&s.governs),
        url: &s.url,
        in_defined_term_set: format!("{origin}/register/"),
    }
}

/// The JSON-LD for one standard's page: its DefinedTerm with a schema.org context.
pub fn defined_term_page(s: &Standard, origin: &str) -> serde_json::Value {
    let mut v = serde_json::to_value(defined_term(s, origin)).unwrap();
    let mut out = serde_json::Map::new();
    out.insert("@context".into(), "https://schema.org".into());
    out.extend(std::mem::take(v.as_object_mut().unwrap()));
    serde_json::Value::Object(out)
}

/// schema.org DefinedTermSet for the whole register.
pub fn defined_term_set(register: &Register, origin: &str) -> serde_json::Value {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "DefinedTermSet",
        "@id": format!("{origin}/register/"),
        "name": "Bundu standards register",
        "description": "The external standards the Bundu, Mukoko and Nyuchi platform builds on, and the status of each.",
        "url": format!("{origin}/register/"),
        "publisher": { "@type": "Organization", "name": "Bundu Foundation", "url": "https://www.bundu.org" },
        "license": "https://creativecommons.org/publicdomain/zero/1.0/",
        "hasDefinedTerm": register.standards.iter().map(|s| defined_term(s, origin)).collect::<Vec<_>>(),
    })
}

#[derive(Serialize)]
struct IndexEntry<'a> {
    id: &'a str,
    name: &'a str,
    domain: &'a str,
    status: &'a str,
    commitment: &'a str,
    current: &'a str,
    steward: Option<&'a str>,
    url: &'a str,
    page: String,
}

#[derive(Serialize)]
struct Index<'a> {
    count: Counts,
    standards: Vec<IndexEntry<'a>>,
}

/// The machine-readable list at /register/index.json, pretty-printed with a final newline,
/// as `JSON.stringify(indexJson(...), null, 2) + "\n"` wrote it.
pub fn index_json(register: &Register, origin: &str) -> String {
    let index = Index {
        count: register.counts,
        standards: register
            .standards
            .iter()
            .map(|s| IndexEntry {
                id: &s.id,
                name: &s.name,
                domain: &s.domain,
                status: &s.status,
                commitment: &s.commitment,
                current: &s.current,
                steward: s.steward.as_deref(),
                url: &s.url,
                page: format!("{origin}/register/{}/", s.id),
            })
            .collect(),
    };
    serde_json::to_string_pretty(&index).unwrap() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"domains":[["b","Second"],["a","First"],["empty","Nothing"]],
      "standards":[
        {"id":"x","name":"X","domain":"a","steward":"S","commitment":"1","current":"1.0",
         "governs":"Line one\n  line two ","status":"adopted","decided":"d","url":"https://x"},
        {"id":"y","name":"Y","domain":"b","commitment":"2","current":"2",
         "governs":"g","status":"proposed","decided":"d","url":"https://y"}]}"#;

    #[test]
    fn keeps_the_domain_order_and_drops_empty_domains() {
        let r = Register::parse(SAMPLE).unwrap();
        let keys: Vec<_> = r.domains.iter().map(|d| d.key.as_str()).collect();
        assert_eq!(keys, ["b", "a"]);
        assert_eq!(
            r.counts,
            Counts {
                total: 2,
                adopted: 1,
                proposed: 1
            }
        );
    }

    #[test]
    fn refuses_a_bad_register() {
        let dup = SAMPLE.replace(r#""id":"y""#, r#""id":"x""#);
        assert!(Register::parse(&dup)
            .unwrap_err()
            .contains("duplicate id x"));
        let unknown = SAMPLE.replace(r#""domain":"b""#, r#""domain":"zz""#);
        assert!(Register::parse(&unknown)
            .unwrap_err()
            .contains("unknown domain zz"));
        let status = SAMPLE.replace("proposed", "maybe");
        assert!(Register::parse(&status).unwrap_err().contains("bad status"));
    }

    #[test]
    fn json_ld_and_index() {
        let r = Register::parse(SAMPLE).unwrap();
        let term = defined_term_page(&r.standards[0], SITE);
        assert_eq!(term["@context"], "https://schema.org");
        assert_eq!(term["description"], "Line one line two");
        assert_eq!(term["@id"], "https://standards.bundu.org/register/x/");
        let set = defined_term_set(&r, SITE);
        assert_eq!(set["hasDefinedTerm"].as_array().unwrap().len(), 2);
        let index = index_json(&r, SITE);
        assert!(index.ends_with("}\n"));
        assert!(index.contains("\"steward\": null"));
        assert!(index.starts_with("{\n  \"count\": {\n    \"total\": 2,"));
    }
}
