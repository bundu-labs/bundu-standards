//! The accessibility rules of the stylesheet, computed from its own tokens.
//!
//! The Bundu floor (standards.bundu.org/bundu-floor/): 7:1 for text in both themes, a 17px body
//! on touch and 15px with a fine pointer, 1.5 leading, 14px captions, a 17px inset, 9px between
//! controls, and hit regions of 45px (touch) and 24px (fine pointer). These are the checks the
//! retired JavaScript tests made (contrast pairs, visible focus, wrapping, `.sr-only`, hit
//! regions), made on the stylesheet that is shipped, plus reduced motion, forced colours and the
//! floors. [`audit`] returns one message per problem.

use std::collections::HashMap;

/// The block `{ ... }` that opens at `open` (the index of its `{`), without its braces.
fn block(css: &str, open: usize) -> &str {
    let mut depth = 0;
    for (i, c) in css[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &css[open + 1..open + i];
                }
            }
            _ => {}
        }
    }
    &css[open + 1..]
}

/// `--name: value;` pairs of a block (colours and lengths alike).
fn declarations(block: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for part in block.split(';') {
        let part = part.split("*/").last().unwrap_or(part);
        if let Some((k, v)) = part.trim().split_once(':') {
            if let Some(name) = k.trim().strip_prefix("--") {
                out.insert(
                    name.to_owned(),
                    v.split("/*").next().unwrap().trim().to_owned(),
                );
            }
        }
    }
    out
}

/// The light tokens, the dark tokens (light's, overridden) and the desktop tier's tokens.
pub fn themes(
    css: &str,
) -> (
    HashMap<String, String>,
    HashMap<String, String>,
    HashMap<String, String>,
) {
    let light = declarations(block(css, css.find(":root {").expect(":root") + 6));
    let dark_at = css
        .find("@media (prefers-color-scheme: dark)")
        .expect("dark scheme");
    let dark_media = block(css, css[dark_at..].find('{').unwrap() + dark_at);
    let mut dark = light.clone();
    dark.extend(declarations(block(
        dark_media,
        dark_media.find('{').unwrap(),
    )));
    let desk_at = css.find("@media (pointer: fine)").expect("desktop tier");
    let desk_media = block(css, css[desk_at..].find('{').unwrap() + desk_at);
    let desk = declarations(block(desk_media, desk_media.find('{').unwrap()));
    (light, dark, desk)
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

fn px(v: &str) -> Option<f64> {
    v.strip_suffix("px")?.parse().ok()
}

/// The rule bodies whose selector list contains `selector` exactly (as one of its selectors).
fn rules<'a>(css: &'a str, selector: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = css[from..].find('{') {
        let open = from + i;
        let head_start = css[..open].rfind(['}', '{', ';']).map_or(0, |j| j + 1);
        let head = css[head_start..open].trim();
        let body = block(css, open);
        if head.split(',').any(|s| s.trim() == selector) {
            out.push(body);
        }
        // Step into at-rules, over plain rules.
        from = if head.starts_with('@') {
            open + 1
        } else {
            open + body.len() + 2
        };
    }
    out
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |e| &rest[i + e + 2..]);
    }
    out.push_str(rest);
    out
}

/// Problems with the stylesheet; empty when it keeps every rule.
pub fn audit(css: &str) -> Vec<String> {
    let stripped = strip_comments(css);
    let css = stripped.as_str();
    let mut p = Vec::new();
    let (light, dark, desk) = themes(css);

    // Text colours, in both themes, on every surface they sit on. APCA Lc by use (Mzizi's
    // bars: body text 75, UI controls and large text 60, non-text UI 30), and WCAG 2 as well,
    // since Mzizi holds a colour to both (body 7:1).
    for (name, t) in [("light", &light), ("dark", &dark)] {
        for (fg, bg, lc, ratio) in [
            ("fg", "bg", 75.0, 7.0),
            ("fg", "chip", 75.0, 7.0),
            ("muted", "bg", 75.0, 7.0),
            ("muted", "chip", 75.0, 7.0),
            ("accent", "bg", 75.0, 7.0),
            ("on-copper-container", "copper-container", 75.0, 7.0),
            // The edge of a control (.copy select, .copy button) and the focus ring.
            ("muted", "bg", 60.0, 3.0),
            ("accent", "bg", 60.0, 3.0),
        ] {
            let (Some(f), Some(b)) = (t.get(fg), t.get(bg)) else {
                p.push(format!("{name}: --{fg} or --{bg} is missing"));
                continue;
            };
            let l = apca(f, b).abs();
            if l < lc {
                p.push(format!(
                    "{name}: --{fg} on --{bg} is APCA Lc {l:.1}, below {lc}"
                ));
            }
            let c = contrast(f, b);
            if c < ratio {
                p.push(format!(
                    "{name}: --{fg} on --{bg} is {c:.2}:1, below {ratio}:1"
                ));
            }
        }
    }
    if !css.contains("color-scheme: light dark") {
        p.push("color-scheme: light dark is missing".into());
    }

    // The floors, in px.
    let floor = |m: &HashMap<String, String>, k: &str| m.get(k).and_then(|v| px(v));
    for (k, want) in [
        ("hit", 45.0),
        ("body", 17.0),
        ("caption", 14.0),
        ("inset-floor", 17.0),
        ("gap-floor", 9.0),
    ] {
        if floor(&light, k) != Some(want) {
            p.push(format!("--{k} must be {want}px on touch"));
        }
    }
    if floor(&desk, "hit") != Some(24.0) || floor(&desk, "body") != Some(15.0) {
        p.push("the desktop tier must be --hit 24px and --body 15px".into());
    }
    if light.get("leading-floor").map(String::as_str) != Some("1.5") {
        p.push("--leading-floor must be 1.5".into());
    }
    if !css.contains("line-height: var(--leading-floor)") && !css.contains("/ var(--leading-floor)")
    {
        p.push("the body does not use --leading-floor".into());
    }

    // Focus is always visible.
    if !rules(css, ":focus-visible")
        .iter()
        .any(|b| b.contains("outline: 3px solid var(--accent)"))
    {
        p.push(":focus-visible must draw a 3px accent outline".into());
    }
    if !css.contains("forced-colors: active") || !css.contains("outline-color: Highlight") {
        p.push("forced colours: focus must use Highlight".into());
    }
    for (i, _) in css.match_indices("outline: none") {
        let open = css[..i].rfind('{').unwrap_or(0);
        let head_start = css[..open].rfind(['}', '{']).map_or(0, |j| j + 1);
        let head = css[head_start..open].trim();
        // Only a programmatic focus target (tabindex=-1) may lose its outline.
        let ok = head.starts_with("main:focus") || head.contains("[tabindex=\"-1\"]:focus");
        if !ok {
            p.push(format!("outline removed on `{head}`"));
        }
    }
    if !rules(css, ".skip:focus").iter().any(|b| b.contains("top:")) {
        p.push("the skip link must become visible on focus".into());
    }

    // Long text wraps; the screen-reader-only class hides visually and no further.
    if !css.contains("overflow-wrap: anywhere") {
        p.push("long text must wrap (overflow-wrap: anywhere)".into());
    }
    if !rules(css, ".sr-only").iter().any(|b| b.contains("clip")) {
        p.push(".sr-only must clip".into());
    }

    // Hit regions: every control and link that is a target.
    for sel in [
        ".copy select",
        ".copy button",
        "header.site a",
        "nav.tabs a",
        "ul.toc a",
        "ul.files a",
        "ul.terms a",
    ] {
        if !rules(css, sel)
            .iter()
            .any(|b| b.contains("min-height: var(--hit)"))
        {
            p.push(format!("`{sel}` must have min-height: var(--hit)"));
        }
    }

    // Reduced motion.
    let rm = css
        .find("prefers-reduced-motion: reduce")
        .map(|i| block(css, css[i..].find('{').unwrap() + i));
    match rm {
        Some(b)
            if b.contains("animation: none")
                && b.contains("transition: none")
                && b.contains("scroll-behavior: auto") => {}
        _ => p.push(
            "prefers-reduced-motion must stop animation, transition and smooth scroll".into(),
        ),
    }

    // No text below the caption floor: a font-size is a max() with the floor, a var, or large.
    for (i, _) in css.match_indices("font-size:") {
        let v = css[i + 10..].split(';').next().unwrap().trim();
        let small = |u: &str| {
            v.strip_suffix(u)
                .and_then(|n| n.parse::<f64>().ok())
                .is_some_and(|n| (u == "px" && n < 14.0) || (u != "px" && n < 0.875))
        };
        if small("px") || small("rem") || small("em") {
            p.push(format!("font-size: {v} is below the 14px caption floor"));
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_stylesheet_keeps_every_rule() {
        assert_eq!(audit(crate::CSS), Vec::<String>::new());
    }

    #[test]
    fn apca_matches_the_reference_values() {
        // The APCA-W3 reference: black on white Lc 106.04, white on black Lc -107.88.
        assert!((apca("#000000", "#ffffff") - 106.04).abs() < 0.1);
        assert!((apca("#ffffff", "#000000") + 107.88).abs() < 0.1);
        assert_eq!(apca("#777777", "#777777"), 0.0);
        // #888 on white is the classic mid grey: about Lc 63.
        assert!((apca("#888888", "#ffffff") - 63.1).abs() < 0.5);
    }

    #[test]
    fn contrast_is_wcag() {
        assert!((contrast("#000000", "#ffffff") - 21.0).abs() < 0.01);
        assert!(contrast("#777777", "#ffffff") < 4.6);
    }

    #[test]
    fn each_rule_catches_its_fault() {
        let css = crate::CSS;
        let cases = [
            ("contrast", css.replacen("--muted: #4d4a45", "--muted: #999999", 1)),
            ("apca", css.replacen("--accent: #7e3b24", "--accent: #a2512f", 1)),
            ("focus", css.replacen("outline: 3px solid var(--accent); outline-offset: 3px; border-radius: 3px; }\nmain:focus", "outline: 1px dotted red; }\nmain:focus", 1)),
            ("hit", css.replacen("--hit: 45px", "--hit: 30px", 1)),
            ("wrap", css.replace("overflow-wrap: anywhere", "")),
            ("motion", css.replace("animation: none !important", "")),
            ("outline", format!("{css}\na.x:focus {{ outline: none; }}")),
            ("small text", format!("{css}\nsmall {{ font-size: 11px; }}")),
        ];
        for (name, bad) in cases {
            assert!(!audit(&bad).is_empty(), "{name} was not caught");
        }
    }
}
