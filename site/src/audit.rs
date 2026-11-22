//! The accessibility rules both sites keep, checked on a rendered page.
//!
//! [`audit`] returns one message per problem, so a test can print them all. It reads the HTML
//! with a small tokenizer rather than a parser: the pages are generated, well formed and small.
//! The rules are the ones the retired JavaScript tests kept (landmarks, skip link, labelled
//! navigation, `lang`, empty `alt` on decorative images, `Status:` text beside colour, the copy
//! control, a CSP that allows no inline code) and the WCAG 2.2 checks that can be made on markup
//! alone (heading order, names for links, buttons and fields, unique ids, header cells (with scope) on tables).

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct Tag {
    name: String,
    attrs: HashMap<String, String>,
    closing: bool,
}

/// Every tag with the text before it, plus the text after the last tag.
fn tokens(html: &str) -> Vec<(String, Option<Tag>)> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        let text = &rest[..i];
        let after = &rest[i..];
        if let Some(body) = after.strip_prefix("<!--") {
            let end = body.find("-->").map_or(body.len(), |e| e + 3);
            out.push((text.to_owned(), None));
            rest = &body[end.min(body.len())..];
            continue;
        }
        let Some(end) = after.find('>') else { break };
        let inner = &after[1..end];
        rest = &after[end + 1..];
        let tag = parse_tag(inner);
        // Raw text elements: skip to the matching close, keeping the body as "text" only for
        // title; scripts and styles are checked by their tags, not their content.
        let raw = tag
            .as_ref()
            .filter(|t| !t.closing && (t.name == "script" || t.name == "style"))
            .map(|t| t.name.clone());
        out.push((text.to_owned(), tag));
        if let Some(name) = raw {
            let close = format!("</{name}>");
            if let Some(e) = rest.find(&close) {
                out.push((String::new(), None));
                rest = &rest[e..];
            }
        }
    }
    out.push((rest.to_owned(), None));
    out
}

fn parse_tag(inner: &str) -> Option<Tag> {
    let (closing, inner) = match inner.strip_prefix('/') {
        Some(r) => (true, r),
        None => (false, inner),
    };
    if inner.starts_with('!') {
        return None;
    }
    let inner = inner.trim_end_matches('/');
    let name_end = inner.find(char::is_whitespace).unwrap_or(inner.len());
    let name = inner[..name_end].to_ascii_lowercase();
    let mut attrs = HashMap::new();
    let mut s = &inner[name_end..];
    loop {
        s = s.trim_start();
        if s.is_empty() {
            break;
        }
        let key_end = s
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(s.len());
        let key = s[..key_end].to_ascii_lowercase();
        s = s[key_end..].trim_start();
        let mut value = String::new();
        if let Some(r) = s.strip_prefix('=') {
            let r = r.trim_start();
            if let Some(q) = r.chars().next().filter(|c| *c == '"' || *c == '\'') {
                let body = &r[1..];
                let e = body.find(q).unwrap_or(body.len());
                value = body[..e].to_owned();
                s = &body[(e + 1).min(body.len())..];
            } else {
                let e = r.find(char::is_whitespace).unwrap_or(r.len());
                value = r[..e].to_owned();
                s = &r[e..];
            }
        }
        if !key.is_empty() {
            attrs.insert(key, value);
        }
    }
    Some(Tag {
        name,
        attrs,
        closing,
    })
}

/// Problems with a rendered page; empty when it keeps every rule. `html` is the whole document.
pub fn audit(html: &str) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut bad = |m: String| p.push(m);
    if !html.starts_with("<!doctype html>") {
        bad("no <!doctype html>".into());
    }
    if !html.contains("<html lang=\"en\">") {
        bad("the html element has no lang".into());
    }
    let toks = tokens(html);
    let mut ids: HashSet<String> = HashSet::new();
    let mut dup = Vec::new();
    let mut refs: Vec<(String, String)> = Vec::new(); // (what, id)
    let mut labels_for: HashSet<String> = HashSet::new();
    let mut fields: Vec<String> = Vec::new();
    let (mut h1s, mut mains, mut footers, mut banners) = (0, 0, 0, 0);
    let mut last_heading = 0u8;
    let mut first_link_seen = false;
    let mut in_body = false;
    let mut in_title = false;
    let mut title = String::new();
    // The open link or button: its kind and the text and image names it has gathered.
    let mut named: Option<(String, String)> = None;
    let mut viewport = false;
    let mut description = false;
    let mut canonical = false;
    let mut noindex = false;
    let mut tables_without_header = 0;
    let mut in_table: Option<bool> = None; // has a header cell yet
    let mut cur_head: Option<(u8, String)> = None;
    let mut headings: Vec<(u8, String)> = Vec::new();
    let mut brand_open = false;
    let mut brand = String::new();

    for (text, tag) in &toks {
        if in_title {
            title.push_str(text);
        }
        if let Some((_, n)) = named.as_mut() {
            n.push_str(text.trim());
        }
        if let Some((_, h)) = cur_head.as_mut() {
            h.push_str(text);
        }
        if brand_open {
            brand.push_str(text);
        }
        let Some(t) = tag else { continue };
        if t.closing {
            match t.name.as_str() {
                "title" => in_title = false,
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    if let Some(h) = cur_head.take() {
                        headings.push(h);
                    }
                }
                "a" | "button" => {
                    brand_open = false;
                    if let Some((kind, name)) = named.take() {
                        if name.trim().is_empty() {
                            bad(format!("a {kind} with no accessible name"));
                        }
                    }
                }
                "table" => {
                    if in_table == Some(false) {
                        tables_without_header += 1;
                    }
                    in_table = None;
                }
                _ => {}
            }
            continue;
        }
        let a = &t.attrs;
        if let Some(id) = a.get("id") {
            if !ids.insert(id.clone()) {
                dup.push(id.clone());
            }
        }
        for k in a.keys() {
            if k.starts_with("on") && k.len() > 2 {
                bad(format!("inline event handler {k} on <{}>", t.name));
            }
        }
        if a.contains_key("style") {
            bad(format!(
                "a style attribute on <{}> (the CSP forbids inline style)",
                t.name
            ));
        }
        if let Some(v) = a.get("aria-labelledby") {
            refs.push(("aria-labelledby".into(), v.clone()));
        }
        match t.name.as_str() {
            "body" => in_body = true,
            "title" => in_title = true,
            "meta" => match a.get("name").map(String::as_str) {
                Some("viewport") => {
                    viewport = true;
                    let c = a.get("content").map_or("", String::as_str);
                    if c.contains("user-scalable=no") || c.contains("maximum-scale=1") {
                        bad("the viewport forbids zoom (WCAG 1.4.4)".into());
                    }
                }
                Some("robots") => noindex = a.get("content").is_some_and(|c| c.contains("noindex")),
                Some("description") => {
                    description = a.get("content").is_some_and(|c| !c.trim().is_empty())
                }
                _ => {}
            },
            "link" if a.get("rel").map(String::as_str) == Some("canonical") => canonical = true,
            "style" => bad("a <style> element (the CSP forbids inline style)".into()),
            "script" => match a.get("src") {
                Some(src) if src.starts_with('/') => {}
                Some(src) => bad(format!("a script from outside this origin: {src}")),
                None => {
                    let ty = a.get("type").map_or("", String::as_str);
                    if ty != "application/ld+json" && ty != "application/json" {
                        bad("an inline script that would run".into());
                    }
                }
            },
            "main" => {
                mains += 1;
                if a.get("id").map(String::as_str) != Some("main")
                    || a.get("tabindex").map(String::as_str) != Some("-1")
                {
                    bad("main must be id=main tabindex=-1, the skip link's target".into());
                }
            }
            "header" if a.get("class").map(String::as_str) == Some("site") => banners += 1,
            "footer" => footers += 1,
            "nav" => {
                if !a.contains_key("aria-label") && !a.contains_key("aria-labelledby") {
                    bad("a nav with no label".into());
                }
            }
            "img" => match a.get("alt") {
                None => bad("an img with no alt".into()),
                Some(alt) => {
                    if let Some((_, n)) = named.as_mut() {
                        n.push_str(alt);
                    }
                }
            },
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = t.name.as_bytes()[1] - b'0';
                if level == 1 {
                    h1s += 1;
                }
                if last_heading == 0 && level != 1 {
                    bad(format!("the first heading is h{level}, not h1"));
                }
                if last_heading != 0 && level > last_heading + 1 {
                    bad(format!(
                        "heading order skips from h{last_heading} to h{level}"
                    ));
                }
                last_heading = level;
                cur_head = Some((level, String::new()));
            }
            "a" => {
                if a.get("class")
                    .is_some_and(|c| c.split(' ').any(|c| c == "brand"))
                {
                    brand_open = true;
                }
                if in_body && !first_link_seen {
                    first_link_seen = true;
                    if a.get("class").map(String::as_str) != Some("skip")
                        || a.get("href").map(String::as_str) != Some("#main")
                    {
                        bad("the first link in the body is not the skip link to #main".into());
                    }
                }
                if let Some(h) = a.get("href") {
                    if let Some(id) = h.strip_prefix('#') {
                        if !id.is_empty() {
                            refs.push(("href".into(), id.to_owned()));
                        }
                    }
                } else {
                    bad("an a with no href".into());
                }
                let mut name = a.get("aria-label").cloned().unwrap_or_default();
                name.push_str(a.get("title").map_or("", |_| ""));
                named = Some(("link".into(), name));
            }
            "button" => {
                named = Some((
                    "button".into(),
                    a.get("aria-label").cloned().unwrap_or_default(),
                ));
            }
            "label" => {
                if let Some(f) = a.get("for") {
                    labels_for.insert(f.clone());
                }
            }
            "select" | "input" | "textarea" => {
                if a.get("type").map(String::as_str) != Some("hidden") {
                    match a.get("id") {
                        Some(id) => fields.push(id.clone()),
                        None if !a.contains_key("aria-label") => {
                            bad(format!("a {} with no id to label", t.name))
                        }
                        None => {}
                    }
                }
            }
            "table" => in_table = Some(false),
            "th" => {
                in_table = in_table.map(|_| true);
                if !a.contains_key("scope") {
                    bad("a th with no scope".into());
                }
            }
            _ => {}
        }
    }
    // Keyboard help is a requirement (proposed Bundu floor): on every page, in the footer, visible,
    // reached with Tab. It names the keys the page needs (Tab, Shift+Tab, Enter) and links to
    // the accessibility statement, which is where a barrier is reported. The page defines no
    // shortcut of its own (so none can lack help or clash with the browser's or a screen
    // reader's keys): `accesskey` is forbidden.
    let footer = html
        .find("<footer")
        .zip(html.find("</footer>"))
        .map(|(a, b)| &html[a..b])
        .unwrap_or("");
    for needle in ["<kbd>Tab</kbd>", "<kbd>Shift</kbd>", "<kbd>Enter</kbd>"] {
        if !footer.contains(needle) {
            bad(format!("the footer's keyboard help lacks {needle}"));
        }
    }
    if !footer.contains("accessibility") {
        bad("the footer's keyboard help does not link to the accessibility statement".into());
    }
    if html.contains(" accesskey=") {
        bad(
            "accesskey defines a shortcut that can clash with the browser's or a screen reader's"
                .into(),
        );
    }
    for m in repetition(&title, &brand, &headings) {
        bad(m);
    }
    for id in dup {
        bad(format!("duplicate id {id}"));
    }
    for (what, id) in refs {
        if !ids.contains(&id) {
            bad(format!("{what} points at #{id}, which does not exist"));
        }
    }
    for f in fields {
        if !labels_for.contains(&f) {
            bad(format!("the field #{f} has no label"));
        }
    }
    if !ids.contains("main") {
        bad("no #main for the skip link".into());
    }
    if title.trim().is_empty() {
        bad("an empty title".into());
    }
    if !viewport {
        bad("no viewport meta".into());
    }
    if !description {
        bad("no meta description".into());
    }
    if !canonical && !noindex {
        bad("no canonical link (or noindex)".into());
    }
    if h1s != 1 {
        bad(format!("{h1s} h1 elements, want exactly 1"));
    }
    if mains != 1 {
        bad(format!("{mains} main elements, want exactly 1"));
    }
    if banners != 1 {
        bad(format!("{banners} site headers (banner), want exactly 1"));
    }
    if footers != 1 {
        bad(format!("{footers} footers, want exactly 1"));
    }
    if tables_without_header > 0 {
        bad(format!(
            "{tables_without_header} table(s) with no header cells"
        ));
    }
    p
}

/// Lower-case words of a text: letters and digits only.
fn words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Words that carry no meaning of their own, and the org's name, which every site of the estate
/// starts with: "Bundu floor — Bundu standards" is not a repeat, "Standards — Bundu standards" is.
const NEUTRAL: [&str; 9] = ["the", "of", "and", "a", "an", "for", "to", "in", "bundu"];

/// Repetition between a page's title, the site name (the brand in the header) and its headings.
///
/// - The title is `<page> — <site>` (or just `<site>` on the home page); the site half is the
///   brand, and no word of it is repeated in the page half (the shared org name aside).
/// - The `h1` does not repeat the site name, except on the home page, which is the site.
/// - A heading does not restate its parent heading.
pub fn repetition(title: &str, brand: &str, headings: &[(u8, String)]) -> Vec<String> {
    let mut p = Vec::new();
    let norm = |s: &str| words(s).join(" ");
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let brand = brand.split_whitespace().collect::<Vec<_>>().join(" ");
    let home = norm(&title) == norm(&brand);
    match title.rsplit_once(" — ") {
        Some((page, site)) => {
            if norm(site) != norm(&brand) {
                p.push(format!(
                    "the title's site half {site:?} is not the brand {brand:?}"
                ));
            }
            let site_words: Vec<String> = words(site)
                .into_iter()
                .filter(|w| !NEUTRAL.contains(&w.as_str()))
                .collect();
            for w in words(page) {
                if site_words.contains(&w) {
                    p.push(format!("the title {title:?} repeats {w:?} in both halves"));
                }
            }
        }
        None if !home => p.push(format!(
            "the title {title:?} is neither the site name nor \"<page> — <site>\""
        )),
        None => {}
    }
    let site_phrase = norm(&brand);
    let mut stack: Vec<(u8, String)> = Vec::new();
    for (level, text) in headings {
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let n = norm(&text);
        if *level == 1 && !home && !site_phrase.is_empty() && n.contains(&site_phrase) {
            p.push(format!("the h1 {text:?} repeats the site name {brand:?}"));
        }
        while stack.last().is_some_and(|(l, _)| l >= level) {
            stack.pop();
        }
        if let Some((_, parent)) = stack.last() {
            if parent.len() >= 4 && n.contains(parent.as_str()) {
                p.push(format!(
                    "the heading {text:?} restates its parent {parent:?}"
                ));
            }
        }
        stack.push((*level, n));
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>T — Home</title><meta name="description" content="d"><link rel="canonical" href="https://x/"><script type="application/json" id="page-json">{"a":1}</script><script src="/copy.js" defer></script></head><body><a class="skip" href="#main">Skip</a><header class="site"><a class="brand" href="/"><img alt="" src="/i.png">Home</a><nav aria-label="Main"><ul><li><a href="/x">X</a></li></ul></nav></header><main id="main" tabindex="-1"><h1>H</h1><h2>S</h2><label for="f">F</label><select id="f"></select><button type="button">Go</button><table><caption>C</caption><tr><th scope="row">a</th></tr></table></main><footer class="site"><p>Every page works with a keyboard alone: <kbd>Tab</kbd> moves forward, <kbd>Shift</kbd>+<kbd>Tab</kbd> moves back, <kbd>Enter</kbd> follows a link. Read the <a href="/accessibility/">accessibility statement</a>.</p></footer></body></html>"##;

    #[test]
    fn a_good_page_has_no_problems() {
        assert_eq!(audit(GOOD), Vec::<String>::new());
    }

    #[test]
    fn each_rule_catches_its_fault() {
        let cases: [(&str, String); 16] = [
            ("lang", GOOD.replace(" lang=\"en\"", "")),
            ("alt", GOOD.replace(" alt=\"\"", "")),
            ("heading", GOOD.replace("<h2>S</h2>", "<h3>S</h3>")),
            ("h1", GOOD.replace("<h1>H</h1>", "")),
            (
                "skip",
                GOOD.replace("class=\"skip\" href=\"#main\"", "href=\"#main\""),
            ),
            ("nav", GOOD.replace(" aria-label=\"Main\"", "")),
            ("name", GOOD.replace(">X</a>", "></a>")),
            ("style", GOOD.replace("<h1>", "<h1 style=\"x:y\">")),
            (
                "script",
                GOOD.replace(
                    "<script src=\"/copy.js\" defer></script>",
                    "<script>run()</script>",
                ),
            ),
            ("label", GOOD.replace("<label for=\"f\">F</label>", "")),
            (
                "table header",
                GOOD.replace("<th scope=\"row\">a</th>", "<td>a</td>"),
            ),
            ("scope", GOOD.replace(" scope=\"row\"", "")),
            ("zoom", GOOD.replace("initial-scale=1", "maximum-scale=1")),
            ("id", GOOD.replace("id=\"f\"", "id=\"main\"")),
            ("keyboard help", GOOD.replace("<kbd>Shift</kbd>", "Shift")),
            ("accesskey", GOOD.replace("<h1>", "<h1 accesskey=\"h\">")),
        ];
        for (name, html) in cases {
            assert!(!audit(&html).is_empty(), "{name} was not caught");
        }
    }

    #[test]
    fn repetition_between_title_and_headings_is_caught() {
        let h = |l: u8, s: &str| (l, s.to_owned());
        let ok = |title: &str, brand: &str, hs: &[(u8, String)]| repetition(title, brand, hs);
        // Fine: a page, a site, an h1 that is the page, headings that add something.
        assert!(ok(
            "Register — Bundu standards",
            "Bundu standards",
            &[h(1, "Register"), h(2, "On this page")]
        )
        .is_empty());
        // The home page is the site: title and h1 may be its name.
        assert!(ok(
            "Bundu standards",
            "Bundu standards",
            &[h(1, "Bundu standards")]
        )
        .is_empty());
        // The org's name starts every site, so a standard called "Bundu floor" is not a repeat.
        assert!(ok(
            "Bundu floor — Bundu standards",
            "Bundu standards",
            &[h(1, "Bundu floor")]
        )
        .is_empty());
        // Caught: the same words on both sides of the title.
        assert!(!ok(
            "Standards — Bundu standards",
            "Bundu standards",
            &[h(1, "Standards")]
        )
        .is_empty());
        // Caught: the site half is not the brand.
        assert!(!ok(
            "Register — Standards",
            "Bundu standards",
            &[h(1, "Register")]
        )
        .is_empty());
        // Caught: no site half on a page that is not home.
        assert!(!ok("Register", "Bundu standards", &[h(1, "Register")]).is_empty());
        // Caught: the h1 repeats the site name.
        assert!(!ok(
            "Register — Bundu standards",
            "Bundu standards",
            &[h(1, "Register of Bundu standards")]
        )
        .is_empty());
        // Caught: a heading restates its parent.
        assert!(!ok(
            "Family — Bundu vocabulary",
            "Bundu vocabulary",
            &[
                h(1, "Family"),
                h(2, "Family tree"),
                h(3, "Using the family tree")
            ]
        )
        .is_empty());
    }
}
