//! The accessibility and token rules of the stylesheet, checked on what Lightning CSS parses.
//!
//! Two things are read, both with the `lightningcss` crate (the one that builds the sheet):
//!
//! - `styles/page.css`, the page styles as written: every declaration is walked, and a literal
//!   colour, length, time or bare number anywhere in it fails the audit. Page CSS takes every
//!   value from a token (`var(--token)`); the literals live in the token layer (`tokens.css`,
//!   the Mzizi tokens), nowhere else. It also declares no custom
//!   property of its own.
//! - the bundle `build.rs` makes from it (page CSS plus the tokens it uses), whose token blocks
//!   are resolved for both themes and both pointer tiers and held to the Bundu floor: APCA and
//!   WCAG contrast, hit regions, spacing, type and leading. The floor's numbers come from the
//!   `mzizi-tokens` crate where it has them (45px touch, 14/9/7px spacing, 17px page edge); the
//!   rest (17/15px body, 14px captions, 24px desktop hit region, 1.5 leading, the APCA bars) are
//!   in [`floor`], pending an export upstream (mzizi-registry#747).
//!
//! [`audit_shipped`] returns one message per problem.

use lightningcss::printer::PrinterOptions;
use lightningcss::properties::custom::CustomPropertyName;
use lightningcss::properties::Property;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::traits::ToCss;
use mzizi_tokens::Roles;
use std::collections::HashMap;

/// The bundle `build.rs` makes: page CSS with the tokens it uses, before minifying.
pub const BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/bundle.css"));
/// The page styles as written.
pub const PAGE_SOURCE: &str = include_str!("../styles/page.css");

/// The Bundu floor numbers `mzizi-tokens` does not export yet (BUNDU_FLOOR in mzizi-registry).
pub mod floor {
    /// The minimum |APCA Lc| for body text, and for non-text UI (a control's edge, a focus ring).
    pub const APCA_BODY: f64 = 75.0;
    /// See [`APCA_BODY`].
    pub const APCA_UI: f64 = 30.0;
    /// The minimum WCAG 2 ratio for body text and for non-text UI.
    pub const WCAG_BODY: f64 = 7.0;
    /// See [`WCAG_BODY`].
    pub const WCAG_UI: f64 = 3.0;
    /// Body text on touch and with a fine pointer, and the smallest text there is, in px.
    pub const BODY_TOUCH_PX: f64 = 17.0;
    /// See [`BODY_TOUCH_PX`].
    pub const BODY_FINE_PX: f64 = 15.0;
    /// See [`BODY_TOUCH_PX`].
    pub const CAPTION_PX: f64 = 14.0;
    /// The hit region with a fine pointer (WCAG 2.5.8), in px.
    pub const HIT_FINE_PX: f64 = 24.0;
    /// The body line height.
    pub const LEADING: f64 = 1.5;
}

type Tokens = HashMap<String, String>;

fn parse(css: &str) -> StyleSheet<'_> {
    StyleSheet::parse(css, ParserOptions::default()).expect("the stylesheet parses")
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

/// `name: value` of a declaration as the printer writes it.
fn decl_text(p: &Property<'_>) -> String {
    p.to_css_string(false, PrinterOptions::default())
        .expect("print a declaration")
}

/// The value of a declaration (what follows the first colon).
fn value_of(p: &Property<'_>) -> String {
    let t = decl_text(p);
    t.split_once(':')
        .map_or(t.clone(), |(_, v)| v.trim().to_owned())
}

fn selector_text(rule: &CssRule<'_>) -> Option<String> {
    match rule {
        CssRule::Style(s) => s.selectors.to_css_string(PrinterOptions::default()).ok(),
        _ => None,
    }
}

fn custom_decls(rule: &CssRule<'_>) -> Vec<(String, String)> {
    let CssRule::Style(s) = rule else {
        return vec![];
    };
    s.declarations
        .declarations
        .iter()
        .filter_map(|p| custom_name(p).map(|n| (n.to_owned(), value_of(p))))
        .collect()
}

/// The token blocks of a bundle: light, dark (light with the dark overrides) and the fine-pointer
/// desktop tier (the overrides alone).
fn themes(bundle: &str) -> (Tokens, Tokens, Tokens) {
    let sheet = parse(bundle);
    let mut light = Tokens::new();
    let mut dark = Tokens::new();
    let mut desk = Tokens::new();
    for rule in &sheet.rules.0 {
        match rule {
            CssRule::Style(_) if selector_text(rule).as_deref() == Some(":root") => {
                light.extend(custom_decls(rule));
            }
            CssRule::Style(_)
                if selector_text(rule).as_deref() == Some("[data-theme=\"dark\"]") =>
            {
                dark.extend(custom_decls(rule));
            }
            CssRule::Media(m) => {
                let q = m
                    .query
                    .to_css_string(PrinterOptions::default())
                    .unwrap_or_default();
                for inner in &m.rules.0 {
                    if q.contains("prefers-color-scheme: dark") {
                        dark.extend(custom_decls(inner));
                    } else if q.contains("pointer: fine") {
                        desk.extend(custom_decls(inner));
                    }
                }
            }
            _ => {}
        }
    }
    let mut dark_all = light.clone();
    dark_all.extend(dark);
    (light, dark_all, desk)
}

/// A token's value with every `var(--x)` replaced by what it resolves to.
fn resolve(tokens: &Tokens, name: &str) -> Option<String> {
    fn expand(tokens: &Tokens, value: &str, depth: u8) -> Option<String> {
        if depth > 12 {
            return None;
        }
        let mut out = String::new();
        let mut rest = value;
        while let Some(i) = rest.find("var(") {
            out.push_str(&rest[..i]);
            let tail = &rest[i + 4..];
            let mut level = 1;
            let mut end = tail.len();
            for (j, c) in tail.char_indices() {
                match c {
                    '(' => level += 1,
                    ')' => {
                        level -= 1;
                        if level == 0 {
                            end = j;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let name = tail[..end].split(',').next().unwrap_or("").trim();
            out.push_str(&expand(tokens, tokens.get(name)?, depth + 1)?);
            rest = tail.get(end + 1..).unwrap_or("");
        }
        out.push_str(rest);
        Some(out)
    }
    expand(tokens, tokens.get(name)?, 0)
}

/// A resolved length or number in px (rem and em are 16px), or `None`.
fn number(value: &str) -> Option<f64> {
    let v = value.trim();
    for (unit, scale) in [("px", 1.0), ("rem", 16.0), ("em", 16.0)] {
        if let Some(n) = v.strip_suffix(unit) {
            return n.trim().parse::<f64>().ok().map(|n| n * scale);
        }
    }
    v.parse().ok()
}

fn hex(value: &str) -> Option<String> {
    let i = value.find('#')?;
    let h: String = value[i..].chars().take(7).collect();
    (h.len() == 7 && h[1..].chars().all(|c| c.is_ascii_hexdigit())).then_some(h)
}

fn luminance(hex: &str) -> f64 {
    let c: Vec<f64> = (0..3)
        .map(|i| u8::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f64 / 255.0)
        .map(|v| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// WCAG contrast ratio of two `#rrggbb` colours.
pub fn contrast(a: &str, b: &str) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// APCA-W3 0.1.9 (SAPC 0.98G-4g, sRGB): the lightness contrast Lc of text colour `txt` on
/// background `bg`, positive for dark text on a light ground and negative for light on dark.
/// Mzizi's palette is solved against the same model, `mzizi-tokens` has no function for it.
pub fn apca(txt: &str, bg: &str) -> f64 {
    fn y(hex: &str) -> f64 {
        let c = |i: usize| {
            (u8::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f64 / 255.0).powf(2.4)
        };
        let y = 0.2126729 * c(0) + 0.7151522 * c(1) + 0.0721750 * c(2);
        // Soft clamp of near-black.
        if y < 0.022 {
            y + (0.022 - y).powf(1.414)
        } else {
            y
        }
    }
    let (yt, yb) = (y(txt), y(bg));
    if (yb - yt).abs() < 0.0005 {
        return 0.0;
    }
    let lc = if yb > yt {
        let s = (yb.powf(0.56) - yt.powf(0.57)) * 1.14;
        if s < 0.1 {
            0.0
        } else {
            s - 0.027
        }
    } else {
        let s = (yb.powf(0.65) - yt.powf(0.62)) * 1.14;
        if s > -0.1 {
            0.0
        } else {
            s + 0.027
        }
    };
    lc * 100.0
}

/// The literal values in a declaration's value: colours, lengths, times and bare numbers. Zero,
/// percentages and keywords are not literals; a bare number is allowed inside `calc()`.
pub fn literals(value: &str) -> Vec<String> {
    let mut found = Vec::new();
    let b: Vec<char> = value.chars().collect();
    let mut i = 0;
    let mut calc_depth: Vec<usize> = Vec::new(); // paren depth at each open calc(
    let mut depth = 0usize;
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    while i < b.len() {
        let c = b[i];
        if c == '"' || c == '\'' {
            // A string: its content is text, not a value (a font name, say).
            i += 1;
            while i < b.len() && b[i] != c {
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == '(' {
            depth += 1;
            let before: String = b[..i]
                .iter()
                .rev()
                .take_while(|c| is_ident(**c))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if before == "calc" {
                calc_depth.push(depth);
            }
            if [
                "rgb",
                "rgba",
                "hsl",
                "hsla",
                "hwb",
                "lab",
                "lch",
                "oklab",
                "oklch",
                "color",
                "color-mix",
            ]
            .contains(&before.as_str())
            {
                found.push(format!("{before}()"));
            }
            // A custom property's name is a reference, not a value.
            if before == "var" {
                let mut j = i + 1;
                while j < b.len() && (is_ident(b[j]) || b[j] == ' ') {
                    j += 1;
                }
                i = j;
                continue;
            }
        } else if c == ')' {
            if calc_depth.last() == Some(&depth) {
                calc_depth.pop();
            }
            depth = depth.saturating_sub(1);
        } else if c == '#' {
            let h: String = b[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if matches!(h.len(), 3 | 4 | 6 | 8) {
                found.push(format!("#{h}"));
            }
            i += 1 + h.len();
            continue;
        } else {
            let starts_number = c.is_ascii_digit()
                || (c == '.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
                || ((c == '-' || c == '+')
                    && b.get(i + 1)
                        .is_some_and(|d| d.is_ascii_digit() || *d == '.'));
            let prev_ident =
                i > 0 && is_ident(b[i - 1]) && !(b[i - 1] == '-' && (c == '-' || c == '+'));
            if starts_number && !prev_ident {
                let mut j = i + 1;
                while j < b.len() && (b[j].is_ascii_digit() || b[j] == '.') {
                    j += 1;
                }
                let n: f64 = b[i..j].iter().collect::<String>().parse().unwrap_or(1.0);
                let mut k = j;
                while k < b.len() && (b[k].is_ascii_alphabetic() || b[k] == '%') {
                    k += 1;
                }
                let unit: String = b[j..k].iter().collect();
                let in_calc = !calc_depth.is_empty();
                let ok = n == 0.0 || unit == "%" || (unit.is_empty() && in_calc);
                if !ok {
                    found.push(format!("{}{}", b[i..j].iter().collect::<String>(), unit));
                }
                i = k;
                continue;
            }
        }
        i += 1;
    }
    found
}

/// Every style rule of a rule list, media included, with the media query that holds it.
fn style_rules<'a, 'i>(
    rules: &'a [CssRule<'i>],
    query: &str,
    out: &mut Vec<(String, &'a CssRule<'i>)>,
) {
    for rule in rules {
        match rule {
            CssRule::Style(s) => {
                out.push((query.to_owned(), rule));
                style_rules(&s.rules.0, query, out);
            }
            CssRule::Media(m) => {
                let q = m
                    .query
                    .to_css_string(PrinterOptions::default())
                    .unwrap_or_default();
                style_rules(&m.rules.0, &q, out);
            }
            _ => {}
        }
    }
}

fn declarations<'a, 'i>(rule: &'a CssRule<'i>) -> Vec<&'a Property<'i>> {
    match rule {
        CssRule::Style(s) => s
            .declarations
            .declarations
            .iter()
            .chain(&s.declarations.important_declarations)
            .collect(),
        _ => vec![],
    }
}

/// Problems with page CSS as written: a literal value, or a custom property of its own.
pub fn lint_page(page: &str) -> Vec<String> {
    let sheet = parse(page);
    let mut rules = Vec::new();
    style_rules(&sheet.rules.0, "", &mut rules);
    let mut p = Vec::new();
    for (_, rule) in &rules {
        let sel = selector_text(rule).unwrap_or_default();
        for d in declarations(rule) {
            if let Some(name) = custom_name(d) {
                p.push(format!(
                    "{sel}: declares {name}; page CSS declares no tokens"
                ));
                continue;
            }
            let value = value_of(d);
            for lit in literals(&value) {
                p.push(format!(
                    "{sel} {{ {} }}: the literal {lit}; use a token",
                    decl_text(d)
                ));
            }
        }
    }
    p
}

/// The audit of the stylesheet both sites serve.
pub fn audit_shipped() -> Vec<String> {
    audit(PAGE_SOURCE, BUNDLE)
}

/// Problems with page CSS `page` and its bundle; empty when both keep every rule.
pub fn audit(page: &str, bundle: &str) -> Vec<String> {
    let mut p = lint_page(page);
    let (light, dark, desk) = themes(bundle);

    // Contrast, from the resolved tokens of each theme. APCA Lc by use, and WCAG 2 as well.
    for (name, t) in [("light", &light), ("dark", &dark)] {
        for (fg, bg, lc, ratio) in [
            (
                "--text-primary",
                "--base",
                floor::APCA_BODY,
                floor::WCAG_BODY,
            ),
            (
                "--text-primary",
                "--container",
                floor::APCA_BODY,
                floor::WCAG_BODY,
            ),
            (
                "--text-secondary",
                "--base",
                floor::APCA_BODY,
                floor::WCAG_BODY,
            ),
            (
                "--mineral-copper-text",
                "--base",
                floor::APCA_BODY,
                floor::WCAG_BODY,
            ),
            (
                "--mineral-copper-on-container",
                "--mineral-copper-container",
                floor::APCA_BODY,
                floor::WCAG_BODY,
            ),
            // The edge of a control, and the focus ring.
            ("--border-control", "--base", floor::APCA_UI, floor::WCAG_UI),
            ("--ring", "--base", floor::APCA_UI, floor::WCAG_UI),
        ] {
            let (Some(f), Some(b)) = (
                resolve(t, fg).and_then(|v| hex(&v)),
                resolve(t, bg).and_then(|v| hex(&v)),
            ) else {
                p.push(format!("{name}: {fg} or {bg} does not resolve to a colour"));
                continue;
            };
            let l = apca(&f, &b).abs();
            if l < lc {
                p.push(format!(
                    "{name}: {fg} on {bg} is APCA Lc {l:.1}, below {lc}"
                ));
            }
            let c = contrast(&f, &b);
            if c < ratio {
                p.push(format!("{name}: {fg} on {bg} is {c:.2}:1, below {ratio}:1"));
            }
        }
    }

    // The floor, from the tokens: touch values from `mzizi-tokens`, the rest from `floor`.
    let len = |t: &Tokens, n: &str| resolve(t, n).and_then(|v| number(&v));
    let fine = |n: &str| {
        desk.get(n)
            .and_then(|v| number(v))
            .or_else(|| len(&light, n))
    };
    if len(&light, "--target-floor") != Some(f64::from(Roles::SPACING_TOUCH_MIN)) {
        p.push(format!(
            "--target-floor must be {}px on touch",
            Roles::SPACING_TOUCH_MIN
        ));
    }
    if fine("--target-floor").is_none_or(|v| v < floor::HIT_FINE_PX) {
        p.push(format!(
            "--target-floor must be at least {}px with a fine pointer",
            floor::HIT_FINE_PX
        ));
    }
    for (n, min) in [
        ("--space-control-gap", f64::from(Roles::SPACING_CONTROL_GAP)),
        ("--space-page-x", f64::from(Roles::SPACING_PAGE_X)),
    ] {
        if len(&light, n).is_none_or(|v| v < min) {
            p.push(format!("{n} is below the {min}px floor"));
        }
    }
    if len(&light, "--fs-body").is_none_or(|v| v < floor::BODY_TOUCH_PX) {
        p.push(format!(
            "--fs-body must be at least {}px on touch",
            floor::BODY_TOUCH_PX
        ));
    }
    if fine("--fs-body").is_none_or(|v| v < floor::BODY_FINE_PX) {
        p.push(format!(
            "--fs-body must be at least {}px with a fine pointer",
            floor::BODY_FINE_PX
        ));
    }
    for t in [&light, &dark] {
        if resolve(t, "--lh-body")
            .and_then(|v| v.parse::<f64>().ok())
            .is_none_or(|v| v < floor::LEADING)
        {
            p.push(format!(
                "the body leading must be at least {}",
                floor::LEADING
            ));
        }
    }

    // The page uses the tokens that carry the floor.
    let sheet = parse(page);
    let mut rules = Vec::new();
    style_rules(&sheet.rules.0, "", &mut rules);
    let decls_of = |sel: &str, media: &str| -> Vec<String> {
        rules
            .iter()
            .filter(|(q, r)| {
                q.contains(media)
                    && selector_text(r).is_some_and(|s| s.split(',').any(|x| x.trim() == sel))
            })
            .flat_map(|(_, r)| declarations(r).into_iter().map(decl_text))
            .collect()
    };
    let has = |sel: &str, media: &str, needle: &str| {
        decls_of(sel, media)
            .iter()
            .any(|d| d.to_lowercase().contains(&needle.to_lowercase()))
    };
    for sel in [
        ".copy select",
        ".copy button",
        "header.site a",
        "nav.tabs a",
        "ul.toc a",
        "ul.files a",
        "ul.terms a",
    ] {
        if !has(sel, "", "min-height: var(--target-floor)") {
            p.push(format!("`{sel}` must have min-height: var(--target-floor)"));
        }
    }
    if !has(":focus-visible", "", "var(--ring-width)") || !has(":focus-visible", "", "var(--ring)")
    {
        p.push(
            ":focus-visible must draw the ring: outline var(--ring-width) solid var(--ring)".into(),
        );
    }
    if !has(
        ":focus-visible",
        "forced-colors",
        "outline-color: Highlight",
    ) {
        p.push("forced colours: focus must use Highlight".into());
    }
    for (q, r) in &rules {
        let sel = selector_text(r).unwrap_or_default();
        if declarations(r)
            .iter()
            .any(|d| decl_text(d).contains("outline: none"))
            && !(sel.starts_with("main:focus") || sel.contains("[tabindex=\"-1\"]:focus"))
        {
            p.push(format!("outline removed on `{sel}` {q}"));
        }
    }
    if !has(".skip:focus", "", "top:") {
        p.push("the skip link must become visible on focus".into());
    }
    if !PAGE_HAS_WRAP.iter().any(|n| page.contains(n)) {
        p.push("long text must wrap (overflow-wrap: anywhere)".into());
    }
    if !decls_of(".sr-only", "").iter().any(|d| d.contains("clip")) {
        p.push(".sr-only must clip".into());
    }
    let motion = decls_of("*", "prefers-reduced-motion").join(" ");
    for need in [
        "animation: none",
        "transition: none",
        "scroll-behavior: auto",
    ] {
        if !motion.contains(need) {
            p.push(format!("prefers-reduced-motion must set {need}"));
        }
    }
    if !page.contains("color-scheme: light dark") {
        p.push("color-scheme: light dark is missing".into());
    }
    // Text sizes come from the type tokens, each at or above the caption floor in both tiers.
    let mut used = Vec::new();
    for (_, r) in &rules {
        for d in declarations(r) {
            let t = decl_text(d);
            if t.starts_with("font-size:") || t.starts_with("font:") {
                for part in t.split("var(").skip(1) {
                    let n = part.split(')').next().unwrap_or("");
                    if n.starts_with("--fs-") {
                        used.push(n.to_owned());
                    }
                }
            }
        }
    }
    for n in used {
        for (tier, t) in [("light", &light), ("dark", &dark)] {
            if len(t, &n).is_none_or(|v| v < floor::CAPTION_PX) {
                p.push(format!(
                    "{tier}: {n} is below the {}px caption floor",
                    floor::CAPTION_PX
                ));
            }
        }
    }
    p
}

const PAGE_HAS_WRAP: [&str; 1] = ["overflow-wrap: anywhere"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_stylesheet_keeps_every_rule() {
        assert_eq!(audit_shipped(), Vec::<String>::new());
    }

    #[test]
    fn apca_matches_the_reference_values() {
        // The APCA-W3 reference: black on white Lc 106.04, white on black Lc -107.88.
        assert!((apca("#000000", "#ffffff") - 106.04).abs() < 0.1);
        assert!((apca("#ffffff", "#000000") + 107.88).abs() < 0.1);
        assert_eq!(apca("#777777", "#777777"), 0.0);
        assert!((apca("#888888", "#ffffff") - 63.1).abs() < 0.5);
    }

    #[test]
    fn contrast_is_wcag() {
        assert!((contrast("#000000", "#ffffff") - 21.0).abs() < 0.01);
        assert!(contrast("#777777", "#ffffff") < 4.6);
    }

    #[test]
    fn literals_are_found_and_tokens_are_not() {
        assert_eq!(
            literals("var(--space-base) var(--space-md)"),
            Vec::<String>::new()
        );
        assert_eq!(literals("0"), Vec::<String>::new());
        assert_eq!(
            literals("calc(-1 * var(--border-width))"),
            Vec::<String>::new()
        );
        assert_eq!(literals("rect(0 0 0 0)"), Vec::<String>::new());
        assert_eq!(literals("inset(50%)"), Vec::<String>::new());
        assert_eq!(literals("1px solid var(--border)"), ["1px"]);
        assert_eq!(literals("#fff"), ["#fff"]);
        assert_eq!(literals("rgb(0 0 0 / .4)"), ["rgb()", ".4"]);
        assert_eq!(literals("1.5"), ["1.5"]);
        assert_eq!(literals("0.5rem 12px"), ["0.5rem", "12px"]);
        assert_eq!(literals("200ms"), ["200ms"]);
        assert_eq!(
            literals("\"Noto Sans 12px\", var(--font-family-sans)"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn the_page_css_has_no_literal_and_declares_no_token() {
        assert_eq!(lint_page(PAGE_SOURCE), Vec::<String>::new());
        let bad = PAGE_SOURCE.replacen("var(--base)", "#fafafa", 1);
        assert!(lint_page(&bad).iter().any(|m| m.contains("#fafafa")));
        let bad = PAGE_SOURCE.replacen("var(--space-base)", "16px", 1);
        assert!(lint_page(&bad).iter().any(|m| m.contains("16px")));
        let bad = format!("{PAGE_SOURCE}\n:root {{ --mine: red; }}");
        assert!(lint_page(&bad)
            .iter()
            .any(|m| m.contains("declares --mine")));
    }

    #[test]
    fn each_rule_catches_its_fault() {
        let cases: Vec<(&str, String, String)> = vec![
            (
                "target",
                PAGE_SOURCE.replacen(
                    "min-height: var(--target-floor)",
                    "min-height: var(--space-sm)",
                    1,
                ),
                BUNDLE.to_owned(),
            ),
            (
                "focus",
                PAGE_SOURCE.replace(
                    "outline: var(--ring-width) solid var(--ring);",
                    "outline: 0;",
                ),
                BUNDLE.to_owned(),
            ),
            (
                "wrap",
                PAGE_SOURCE.replace("overflow-wrap: anywhere", ""),
                BUNDLE.to_owned(),
            ),
            (
                "motion",
                PAGE_SOURCE.replace("animation: none !important", ""),
                BUNDLE.to_owned(),
            ),
            (
                "outline",
                format!("{PAGE_SOURCE}\na.x:focus {{ outline: none; }}"),
                BUNDLE.to_owned(),
            ),
            (
                "contrast",
                PAGE_SOURCE.to_owned(),
                BUNDLE.replacen("--text-secondary: #504f4e", "--text-secondary: #999999", 1),
            ),
            (
                "touch floor",
                PAGE_SOURCE.to_owned(),
                BUNDLE.replacen("--target-floor: 2.8125rem", "--target-floor: 2rem", 1),
            ),
            (
                "leading",
                PAGE_SOURCE.to_owned(),
                BUNDLE.replacen("--lh-body: 1.6", "--lh-body: 1.2", 1),
            ),
        ];
        for (name, page, bundle) in cases {
            assert!(!audit(&page, &bundle).is_empty(), "{name} was not caught");
        }
    }
}
