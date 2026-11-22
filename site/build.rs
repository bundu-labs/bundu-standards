//! Builds the one stylesheet both sites serve, with Lightning CSS, from `styles/page.css`.
//!
//! The pipeline is mzizi-registry's (`mzizi-rs/crates/mzizi-css`): the same `lightningcss`
//! crate and version, and the same browser targets. Page CSS `@import`s the token layer
//! (`tokens/tokens.css`, the Mzizi tokens, vendored by `tools/vendor-tokens.sh`); this bundles
//! it into one sheet, drops every custom property no
//! rule uses (directly or through another token), and writes two files to OUT_DIR:
//!
//! - `bundle.css`, the pruned sheet, readable, which `css_audit` parses;
//! - `site.css`, the same, minified for the browser targets: what the sites serve.
//!
//! No JavaScript, no npm: Rust only.

use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::printer::PrinterOptions;
use lightningcss::properties::custom::CustomPropertyName;
use lightningcss::properties::Property;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::ParserOptions;
use lightningcss::targets::{Browsers, Targets};
use std::collections::BTreeSet;
use std::{env, fs, path::Path};

/// The browser floor of mzizi-css: the first versions with `color-mix()`, nesting and `:is()`.
fn targets() -> Targets {
    Targets {
        browsers: Some(Browsers {
            chrome: Some(111 << 16),
            edge: Some(111 << 16),
            firefox: Some(113 << 16),
            safari: Some((16 << 16) | (2 << 8)),
            ..Browsers::default()
        }),
        ..Targets::default()
    }
}

/// The `--names` a declaration's text refers to with `var(`.
fn refs(text: &str, into: &mut BTreeSet<String>) {
    let mut rest = text;
    while let Some(i) = rest.find("var(--") {
        let tail = &rest[i + 4..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(tail.len());
        into.insert(tail[..end].to_owned());
        rest = &tail[end..];
    }
}

fn custom_name<'a>(p: &'a Property<'_>) -> Option<&'a str> {
    match p {
        Property::Custom(c) => match &c.name {
            CustomPropertyName::Custom(id) => Some(id.0.as_ref()),
            CustomPropertyName::Unknown(_) => None,
        },
        _ => None,
    }
}

fn text(p: &Property<'_>) -> String {
    p.to_css_string(false, PrinterOptions::default())
        .expect("print a declaration")
}

/// A rule that only declares custom properties: part of the token layer.
fn is_token_rule(rule: &CssRule<'_>) -> bool {
    matches!(rule, CssRule::Style(s)
        if !s.declarations.declarations.is_empty()
            && s.declarations.declarations.iter().all(|p| custom_name(p).is_some()))
}

/// The names the page rules use: every `var()` in a declaration outside the token layer.
fn used_by_pages(rules: &[CssRule<'_>], into: &mut BTreeSet<String>) {
    for rule in rules {
        match rule {
            r if is_token_rule(r) => {}
            CssRule::Style(s) => {
                for p in s
                    .declarations
                    .declarations
                    .iter()
                    .chain(&s.declarations.important_declarations)
                {
                    refs(&text(p), into);
                }
                used_by_pages(&s.rules.0, into);
            }
            CssRule::Media(m) => used_by_pages(&m.rules.0, into),
            _ => {}
        }
    }
}

/// Add the names the kept tokens refer to, until nothing new is added.
fn close_over_tokens(rules: &[CssRule<'_>], needed: &mut BTreeSet<String>) -> bool {
    let mut grew = false;
    for rule in rules {
        match rule {
            CssRule::Style(s) if is_token_rule(rule) => {
                for p in &s.declarations.declarations {
                    if custom_name(p).is_some_and(|n| needed.contains(n)) {
                        let before = needed.len();
                        refs(&text(p), needed);
                        grew |= needed.len() != before;
                    }
                }
            }
            CssRule::Media(m) => grew |= close_over_tokens(&m.rules.0, needed),
            _ => {}
        }
    }
    grew
}

/// Drop the token declarations nobody needs, then the rules and media that are left empty.
fn prune(rules: &mut Vec<CssRule<'_>>, needed: &BTreeSet<String>) {
    for rule in rules.iter_mut() {
        match rule {
            CssRule::Style(s)
                if !s.declarations.declarations.is_empty()
                    && s.declarations
                        .declarations
                        .iter()
                        .all(|p| custom_name(p).is_some()) =>
            {
                s.declarations
                    .declarations
                    .retain(|p| custom_name(p).is_some_and(|n| needed.contains(n)));
            }
            CssRule::Media(m) => prune(&mut m.rules.0, needed),
            _ => {}
        }
    }
    rules.retain(|r| match r {
        CssRule::Style(s) => {
            !(s.declarations.declarations.is_empty()
                && s.declarations.important_declarations.is_empty()
                && s.rules.0.is_empty())
        }
        CssRule::Media(m) => !m.rules.0.is_empty(),
        _ => true,
    });
}

fn main() {
    println!("cargo:rerun-if-changed=styles");
    println!("cargo:rerun-if-changed=tokens");
    let fs_provider = FileProvider::new();
    let mut bundler = Bundler::new(&fs_provider, None, ParserOptions::default());
    let mut sheet = bundler
        .bundle(Path::new("styles/page.css"))
        .expect("bundle styles/page.css and the token files it imports");

    let mut needed = BTreeSet::new();
    used_by_pages(&sheet.rules.0, &mut needed);
    while close_over_tokens(&sheet.rules.0, &mut needed) {}
    prune(&mut sheet.rules.0, &needed);

    let readable = sheet
        .to_css(PrinterOptions {
            targets: targets(),
            ..PrinterOptions::default()
        })
        .expect("print the bundle")
        .code;
    let minified = sheet
        .to_css(PrinterOptions {
            minify: true,
            targets: targets(),
            ..PrinterOptions::default()
        })
        .expect("print the minified sheet")
        .code;
    let out = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out).join("bundle.css"), readable).unwrap();
    let css = format!("{minified}\n");
    // FNV-1a over the served bytes, as a quoted entity tag.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in css.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    fs::write(Path::new(&out).join("site.css"), css).unwrap();
    fs::write(
        Path::new(&out).join("site.css.etag"),
        format!("\"{h:016x}\""),
    )
    .unwrap();
}
