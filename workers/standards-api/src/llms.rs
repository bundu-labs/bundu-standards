//! `/llms.txt` and `/llms-full.txt`: the site as text, for the machines that read it
//! (the register's `llms_txt`, adopted: the machine index at each site root).
//!
//! `/llms-full.txt` is every page of the site in one file, in the layout docs.bundu.org serves
//! and readers of it parse: a `# Title` line, a blank line, `Source: <the page's URL>`, the
//! page's description, then its body as Markdown. A page's body is what its "Copy this page
//! as Markdown" control gives (the rules of `public/copy.js`, ported below), except on our
//! own standards, whose body is their README as written, with its links made absolute.
//! `/llms.txt` is the index: one line per page, with its URL and description.
//!
//! Both are built once, from the pages the site serves, so they cannot drift from them.

use crate::pages::{self, DOCUMENTS, REGISTER};
use crate::register::SITE;
use std::sync::LazyLock;

/// What both files are served as.
pub const CONTENT_TYPE: &str = "text/plain; charset=utf-8";
/// The site's name, as the page shell's brand.
const NAME: &str = "Bundu standards";
const SUMMARY: &str = "The open standards the Bundu, Mukoko and Nyuchi platform builds on: the register of the external standards it adopts or proposes, and the standards it publishes where none existed.";

/// One page, as text.
pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub body: String,
}

/// The sections of the index and the paths of their pages, in the order of both files.
pub fn sections() -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "Pages",
            ["/", "/register/", "/register/proposed/", "/accessibility/"]
                .map(str::to_owned)
                .to_vec(),
        ),
        (
            "Our standards",
            DOCUMENTS.keys().map(|slug| format!("/{slug}/")).collect(),
        ),
        (
            "Register entries",
            REGISTER
                .standards
                .iter()
                .map(|s| format!("/register/{}/", s.id))
                .collect(),
        ),
    ]
}

static ENTRIES: LazyLock<Vec<(&'static str, Vec<Entry>)>> = LazyLock::new(|| {
    sections()
        .into_iter()
        .map(|(name, paths)| (name, paths.iter().map(|p| entry(p)).collect()))
        .collect()
});

/// `/llms-full.txt`: every page, whole.
pub static FULL: LazyLock<String> = LazyLock::new(|| {
    let mut out = format!("# {NAME} — full text\n\n> {SITE}\n");
    for e in ENTRIES.iter().flat_map(|(_, e)| e) {
        out.push_str(&format!("\n\n---\n\n# {}\n\nSource: {}\n", e.title, e.url));
        if !e.description.is_empty() {
            out.push_str(&format!("\n{}\n", e.description));
        }
        out.push_str(&format!("\n{}\n", e.body.trim()));
    }
    out
});

/// `/llms.txt`: the index of the pages.
pub static INDEX: LazyLock<String> = LazyLock::new(|| {
    let mut out = format!("# {NAME}\n\n> {SUMMARY} Published at {SITE}.\n\n");
    out.push_str(&format!(
        "The text of every page as a single file is at {SITE}{}. The register as JSON is at \
         {SITE}/register/index.json, and every route is described at {SITE}{}.\n",
        pages::LLMS_FULL_PATH,
        pages::OPENAPI_PATH
    ));
    for (name, entries) in ENTRIES.iter() {
        out.push_str(&format!("\n## {name}\n\n"));
        for e in entries {
            let desc = if e.description.is_empty() {
                String::new()
            } else {
                format!(": {}", e.description)
            };
            out.push_str(&format!("- [{}]({}){desc}\n", e.title, e.url));
        }
    }
    out
});

/// One page as text: its title (the page's `h1`), URL, description and body.
fn entry(path: &str) -> Entry {
    let page = pages::render(path).unwrap_or_else(|| panic!("llms: no page at {path}"));
    let doc = html::parse(&page.body);
    let url = format!("{SITE}{path}");
    let description = doc
        .find(&|e| e.tag == "meta" && e.attr("name") == Some("description"))
        .and_then(|e| e.attr("content"))
        .unwrap_or_default()
        .to_owned();
    let main = doc
        .find(&|e| e.tag == "main")
        .unwrap_or_else(|| panic!("llms: {path} has no main"));
    let mut blocks = Vec::new();
    markdown::blocks(&main.children, &mut blocks, &url);
    blocks.retain(|b| !b.is_empty());
    // The page's heading, which carries a standard's status ("H3 (adopted)"); where the
    // heading is shared by two pages (the register's tabs), the page's own title.
    let named = doc
        .find(&|e| e.tag == "title")
        .map(|e| e.text())
        .unwrap_or_default();
    let named = named.strip_suffix(" — Bundu standards").unwrap_or(&named);
    let heading = match blocks.first().and_then(|b| b.strip_prefix("# ")) {
        Some(h) => {
            let h = h.replace('\n', " ");
            blocks.remove(0);
            h
        }
        None => String::new(),
    };
    let title = if heading.starts_with(named) || named.is_empty() {
        heading
    } else {
        named.to_owned()
    };
    let slug = path.trim_matches('/');
    let body = match DOCUMENTS.get(slug) {
        Some(d) => readme_body(slug, d),
        None => blocks.join("\n\n"),
    };
    Entry {
        title,
        url,
        description,
        body,
    }
}

/// One of our own standards: its README, without the `# Title` line the entry already has,
/// with relative links made absolute, then its files, as the page lists them.
fn readme_body(slug: &str, doc: &serde_json::Value) -> String {
    let base = format!("{SITE}/{slug}/");
    let files = doc["files"].as_object();
    let readme = files
        .and_then(|f| f.get("README.md"))
        .and_then(|r| r.as_str())
        .unwrap_or_default()
        .trim_start();
    let readme = match readme.strip_prefix("# ") {
        Some(rest) => rest.split_once('\n').map_or("", |(_, r)| r),
        None => readme,
    };
    let mut out = absolutize(readme.trim(), &base);
    let names: Vec<&String> = files
        .map(|f| f.keys().filter(|k| *k != "README.md").collect())
        .unwrap_or_default();
    if !names.is_empty() {
        out.push_str("\n\n## Files\n\n");
        for f in names {
            out.push_str(&format!("- [`{f}`]({base}{f})\n"));
        }
        out.push_str(&format!(
            "\nThe source of these files, with their tests, is \
             [standards/{slug}/ on GitHub](https://github.com/bundu-labs/bundu-standards/tree/main/standards/{slug})."
        ));
    }
    out
}

/// Markdown with the target of every relative inline link (`[text](target)`) resolved
/// against `base`. A target with a title or a space is left as it is.
fn absolutize(md: &str, base: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut rest = md;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i + 2]);
        rest = &rest[i + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = &rest[..end];
        if !target.is_empty() && !target.contains(char::is_whitespace) {
            out.push_str(&resolve(base, target));
            rest = &rest[end..];
        }
    }
    out.push_str(rest);
    out
}

/// `href` resolved against the absolute URL `base`, as a browser does (`new URL(href, base)`)
/// for the links these pages have: absolute, scheme-relative, root-relative, a query, a
/// fragment, or a path relative to `base`'s directory.
fn resolve(base: &str, href: &str) -> String {
    let scheme = href.find(':').is_some_and(|i| {
        let s = &href[..i];
        s.starts_with(|c: char| c.is_ascii_alphabetic())
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    if scheme {
        return href.to_owned();
    }
    let (origin, path) = match base
        .find("://")
        .and_then(|i| base[i + 3..].find('/').map(|j| i + 3 + j))
    {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, "/"),
    };
    let path = path.split('#').next().unwrap_or("/");
    if let Some(rest) = href.strip_prefix("//") {
        let scheme = &base[..base.find(':').unwrap_or(0)];
        return format!("{scheme}://{rest}");
    }
    if href.starts_with('#') {
        return format!("{origin}{path}{href}");
    }
    let path = path.split('?').next().unwrap_or("/");
    if href.starts_with('?') {
        return format!("{origin}{path}{href}");
    }
    let split = href.find(['?', '#']).unwrap_or(href.len());
    let (href_path, suffix) = href.split_at(split);
    let joined = if href_path.starts_with('/') {
        href_path.to_owned()
    } else {
        format!("{}{href_path}", &path[..=path.rfind('/').unwrap_or(0)])
    };
    format!("{origin}{}{suffix}", normalise(&joined))
}

/// A path with its `.` and `..` segments resolved (RFC 3986, 5.2.4).
fn normalise(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let segments: Vec<&str> = path.split('/').skip(1).collect();
    for (i, s) in segments.iter().enumerate() {
        let last = i + 1 == segments.len();
        match *s {
            "." | ".." => {
                if *s == ".." {
                    out.pop();
                }
                if last {
                    out.push("");
                }
            }
            s => out.push(s),
        }
    }
    format!("/{}", out.join("/"))
}

/// A small HTML reader for the pages this server renders (Dioxus SSR and pulldown-cmark, both
/// well formed): elements, attributes, text and character references. Not a general parser.
mod html {
    pub enum Node {
        Text(String),
        El(El),
    }

    pub struct El {
        pub tag: String,
        pub attrs: Vec<(String, String)>,
        pub children: Vec<Node>,
    }

    const VOID: [&str; 13] = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];

    impl El {
        pub fn attr(&self, name: &str) -> Option<&str> {
            self.attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
        }

        pub fn has(&self, name: &str) -> bool {
            self.attrs.iter().any(|(k, _)| k == name)
        }

        /// The element children.
        pub fn elements(&self) -> impl Iterator<Item = &El> {
            self.children.iter().filter_map(|n| match n {
                Node::El(e) => Some(e),
                Node::Text(_) => None,
            })
        }

        /// The first descendant, in document order, that `pred` holds for.
        pub fn find(&self, pred: &dyn Fn(&El) -> bool) -> Option<&El> {
            for e in self.elements() {
                if pred(e) {
                    return Some(e);
                }
                if let Some(found) = e.find(pred) {
                    return Some(found);
                }
            }
            None
        }

        /// Every descendant, in document order, that `pred` holds for.
        pub fn find_all<'a>(&'a self, pred: &dyn Fn(&El) -> bool, out: &mut Vec<&'a El>) {
            for e in self.elements() {
                if pred(e) {
                    out.push(e);
                }
                e.find_all(pred, out);
            }
        }

        /// All the text inside, as the DOM's `textContent`.
        pub fn text(&self) -> String {
            let mut out = String::new();
            for n in &self.children {
                match n {
                    Node::Text(t) => out.push_str(t),
                    Node::El(e) => out.push_str(&e.text()),
                }
            }
            out
        }
    }

    /// The document as a tree under a root element with no tag.
    pub fn parse(s: &str) -> El {
        let mut stack = vec![El {
            tag: String::new(),
            attrs: Vec::new(),
            children: Vec::new(),
        }];
        let b = s.as_bytes();
        let mut i = 0;
        while i < s.len() {
            let lt = s[i..].find('<').map_or(s.len(), |j| i + j);
            if lt > i {
                push(&mut stack, Node::Text(decode(&s[i..lt])));
            }
            if lt >= s.len() {
                break;
            }
            let rest = &s[lt..];
            if rest.starts_with("<!--") {
                i = rest.find("-->").map_or(s.len(), |j| lt + j + 3);
            } else if rest.starts_with("<!") || rest.starts_with("<?") {
                i = rest.find('>').map_or(s.len(), |j| lt + j + 1);
            } else if let Some(close) = rest.strip_prefix("</") {
                let end = close.find('>').unwrap_or(close.len());
                let name = close[..end].trim().to_ascii_lowercase();
                if let Some(at) = stack.iter().rposition(|e| e.tag == name) {
                    while stack.len() > at.max(1) {
                        let e = stack.pop().unwrap();
                        push(&mut stack, Node::El(e));
                    }
                }
                i = lt + 2 + end + 1;
            } else {
                let mut j = lt + 1;
                while j < s.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-') {
                    j += 1;
                }
                if j == lt + 1 {
                    push(&mut stack, Node::Text("<".to_owned()));
                    i = lt + 1;
                    continue;
                }
                let tag = s[lt + 1..j].to_ascii_lowercase();
                let mut attrs = Vec::new();
                let mut self_closing = false;
                while j < s.len() {
                    match b[j] {
                        c if c.is_ascii_whitespace() => j += 1,
                        b'>' => {
                            j += 1;
                            break;
                        }
                        b'/' if b.get(j + 1) == Some(&b'>') => {
                            self_closing = true;
                            j += 2;
                            break;
                        }
                        b'/' => j += 1,
                        _ => {
                            let start = j;
                            while j < s.len()
                                && !b[j].is_ascii_whitespace()
                                && !matches!(b[j], b'=' | b'>' | b'/')
                            {
                                j += 1;
                            }
                            let name = s[start..j].to_ascii_lowercase();
                            let mut value = String::new();
                            if b.get(j) == Some(&b'=') {
                                j += 1;
                                match b.get(j) {
                                    Some(&q @ (b'"' | b'\'')) => {
                                        let end = s[j + 1..]
                                            .find(q as char)
                                            .map_or(s.len(), |k| j + 1 + k);
                                        value = decode(&s[j + 1..end]);
                                        j = (end + 1).min(s.len());
                                    }
                                    _ => {
                                        let start = j;
                                        while j < s.len()
                                            && !b[j].is_ascii_whitespace()
                                            && b[j] != b'>'
                                        {
                                            j += 1;
                                        }
                                        value = decode(&s[start..j]);
                                    }
                                }
                            }
                            attrs.push((name, value));
                        }
                    }
                }
                i = j;
                let mut el = El {
                    tag,
                    attrs,
                    children: Vec::new(),
                };
                if self_closing || VOID.contains(&el.tag.as_str()) {
                    push(&mut stack, Node::El(el));
                } else if el.tag == "script" || el.tag == "style" {
                    let close = format!("</{}", el.tag);
                    let end = s[i..].find(&close).map_or(s.len(), |k| i + k);
                    el.children.push(Node::Text(s[i..end].to_owned()));
                    i = s[end..].find('>').map_or(s.len(), |k| end + k + 1);
                    push(&mut stack, Node::El(el));
                } else {
                    stack.push(el);
                }
            }
        }
        while stack.len() > 1 {
            let e = stack.pop().unwrap();
            push(&mut stack, Node::El(e));
        }
        stack.pop().unwrap()
    }

    fn push(stack: &mut [El], node: Node) {
        stack.last_mut().unwrap().children.push(node);
    }

    /// Text with its character references (`&amp;`, `&#60;`, `&#x3c;`, ...) replaced.
    pub fn decode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(i) = rest.find('&') {
            out.push_str(&rest[..i]);
            rest = &rest[i..];
            let decoded = rest.find(';').filter(|&e| e <= 10).and_then(|e| {
                let name = &rest[1..e];
                let c = match name {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some('\u{a0}'),
                    _ => match name.strip_prefix('#') {
                        Some(n) => match n.strip_prefix(['x', 'X']) {
                            Some(h) => u32::from_str_radix(h, 16).ok(),
                            None => n.parse().ok(),
                        }
                        .and_then(char::from_u32),
                        None => None,
                    },
                };
                c.map(|c| (c, e))
            });
            match decoded {
                Some((c, e)) => {
                    out.push(c);
                    rest = &rest[e + 1..];
                }
                None => {
                    out.push('&');
                    rest = &rest[1..];
                }
            }
        }
        out.push_str(rest);
        out
    }
}

/// The Markdown of a page's `main`, by the rules of `public/copy.js` (its `blocks`, `flow`,
/// `inline`, `item` and `table`), so a page reads here as its copy control copies it.
mod markdown {
    use super::html::{El, Node};
    use super::resolve;

    const BLOCK: [&str; 15] = [
        "p", "ul", "ol", "dl", "table", "pre", "div", "section", "nav", "h1", "h2", "h3", "h4",
        "h5", "h6",
    ];

    fn is_block(e: &El) -> bool {
        BLOCK.contains(&e.tag.as_str())
    }

    fn skip(e: &El) -> bool {
        e.has("hidden")
            || e.attr("aria-hidden") == Some("true")
            || e.has("data-md-skip")
            || e.tag == "script"
            || e.tag == "style"
    }

    /// Runs of spaces and tabs as one space, no spaces around a line break, trimmed.
    fn tidy(s: &str) -> String {
        let mut spaced = String::with_capacity(s.len());
        for c in s.chars() {
            let c = if c == '\t' { ' ' } else { c };
            if !(c == ' ' && spaced.ends_with(' ')) {
                spaced.push(c);
            }
        }
        spaced
            .split('\n')
            .map(|l| l.trim_matches(' '))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned()
    }

    /// Inline content: text, links, code, emphasis.
    fn inline(nodes: &[Node], base: &str) -> String {
        let mut out = String::new();
        for n in nodes {
            let e = match n {
                Node::Text(t) => {
                    let mut last_space = false;
                    for c in t.chars() {
                        if c.is_whitespace() {
                            if !last_space {
                                out.push(' ');
                            }
                            last_space = true;
                        } else {
                            out.push(c);
                            last_space = false;
                        }
                    }
                    continue;
                }
                Node::El(e) => e,
            };
            if skip(e) {
                continue;
            }
            if let Some(md) = e.attr("data-md") {
                out.push_str(md);
                continue;
            }
            match e.tag.as_str() {
                "br" => out.push('\n'),
                "code" | "kbd" => out.push_str(&format!("`{}`", e.text())),
                "strong" | "b" => {
                    out.push_str(&format!("**{}**", inline(&e.children, base).trim()))
                }
                "em" | "i" => out.push_str(&format!("_{}_", inline(&e.children, base).trim())),
                "a" => {
                    let text = inline(&e.children, base).trim().to_owned();
                    match e.attr("href") {
                        Some(h) if !h.is_empty() && !h.starts_with('#') => {
                            out.push_str(&format!("[{text}]({})", resolve(base, h)));
                        }
                        _ => out.push_str(&text),
                    }
                }
                _ if is_block(e) => out.push_str(&format!(" {} ", inline(&e.children, base))),
                _ => out.push_str(&inline(&e.children, base)),
            }
        }
        out
    }

    /// A list item: its inline parts, with any block parts after an em dash.
    fn item(li: &El, base: &str) -> String {
        let mut parts = Vec::new();
        let mut run = String::new();
        for n in &li.children {
            match n {
                Node::El(e) if is_block(e) && !skip(e) => {
                    if !tidy(&run).is_empty() {
                        parts.push(tidy(&run));
                    }
                    run.clear();
                    let t = tidy(&inline(&e.children, base));
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
                _ => run.push_str(&inline(std::slice::from_ref(n), base)),
            }
        }
        if !tidy(&run).is_empty() {
            parts.push(tidy(&run));
        }
        parts.join(" — ").replace('\n', " ")
    }

    fn cell(c: &El, base: &str) -> String {
        tidy(&inline(&c.children, base))
            .replace('|', "\\|")
            .replace('\n', " ")
    }

    fn cells(row: &El) -> Vec<&El> {
        row.elements()
            .filter(|c| c.tag == "td" || c.tag == "th")
            .collect()
    }

    fn table(t: &El, base: &str) -> String {
        let mut out = Vec::new();
        if let Some(cap) = t.find(&|e| e.tag == "caption") {
            let sr_only = cap
                .attr("class")
                .is_some_and(|c| c.split_whitespace().any(|c| c == "sr-only"));
            if !sr_only {
                out.push(format!("**{}**", tidy(&inline(&cap.children, base))));
                out.push(String::new());
            }
        }
        let mut rows = Vec::new();
        t.find_all(&|e| e.tag == "tr", &mut rows);
        rows.retain(|r| !skip(r));
        let keyed = !t.elements().any(|e| e.tag == "thead")
            && !rows.is_empty()
            && rows
                .iter()
                .all(|r| cells(r).first().is_some_and(|c| c.tag == "th"));
        if keyed {
            // A two-column table of names and values reads better as a list.
            for r in rows {
                let c = cells(r);
                let rest: Vec<String> = c[1..].iter().map(|c| cell(c, base)).collect();
                out.push(format!("- **{}:** {}", cell(c[0], base), rest.join(" ")));
            }
            return out.join("\n");
        }
        for (i, r) in rows.iter().enumerate() {
            let c = cells(r);
            let text: Vec<String> = c.iter().map(|c| cell(c, base)).collect();
            out.push(format!("| {} |", text.join(" | ")));
            if i == 0 {
                out.push(format!("|{}", " --- |".repeat(c.len())));
            }
        }
        out.join("\n")
    }

    /// The blocks of `nodes`, each a paragraph of Markdown.
    pub fn blocks(nodes: &[Node], out: &mut Vec<String>, base: &str) {
        for n in nodes {
            let e = match n {
                Node::Text(t) => {
                    let t = tidy(t);
                    if !t.is_empty() {
                        out.push(t);
                    }
                    continue;
                }
                Node::El(e) => e,
            };
            if skip(e) {
                continue;
            }
            match e.tag.as_str() {
                h @ ("h1" | "h2" | "h3" | "h4" | "h5" | "h6") => {
                    let level = usize::from(h.as_bytes()[1] - b'0');
                    out.push(format!(
                        "{} {}",
                        "#".repeat(level),
                        tidy(&inline(&e.children, base))
                    ));
                }
                "p" | "caption" => out.push(tidy(&inline(&e.children, base))),
                tag @ ("ul" | "ol") => {
                    let items: Vec<String> = e
                        .elements()
                        .filter(|li| li.tag == "li" && !skip(li))
                        .enumerate()
                        .map(|(i, li)| {
                            let mark = if tag == "ol" {
                                format!("{}. ", i + 1)
                            } else {
                                "- ".to_owned()
                            };
                            mark + &item(li, base)
                        })
                        .collect();
                    out.push(items.join("\n"));
                }
                "dl" => {
                    for c in e.elements() {
                        if skip(c) {
                            continue;
                        }
                        match c.tag.as_str() {
                            "dt" => out.push(format!("**{}**", tidy(&inline(&c.children, base)))),
                            "dd" => flow(c, out, base),
                            _ => {}
                        }
                    }
                }
                "table" => out.push(table(e, base)),
                "pre" => out.push(format!("```\n{}\n```", e.text().trim_end_matches('\n'))),
                _ => flow(e, out, base),
            }
        }
    }

    /// An element that may hold paragraphs, or only text.
    fn flow(e: &El, out: &mut Vec<String>, base: &str) {
        if e.elements().any(is_block) {
            blocks(&e.children, out, base);
        } else {
            let t = tidy(&inline(&e.children, base));
            if !t.is_empty() {
                out.push(t);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_full_text_has_the_layout_readers_parse() {
        assert!(FULL.starts_with(&format!(
            "# Bundu standards — full text\n\n> {SITE}\n\n\n---\n\n# Bundu standards\n\nSource: {SITE}/\n\n"
        )));
        assert!(FULL.ends_with('\n') && !FULL.ends_with("\n\n"));
        // Every page: a separator, `# Title`, a blank line, `Source:` and its absolute URL.
        let pages: Vec<&str> = FULL.split("\n\n---\n\n").skip(1).collect();
        let paths: Vec<String> = sections().into_iter().flat_map(|(_, p)| p).collect();
        assert_eq!(pages.len(), paths.len(), "one block per page");
        for (block, path) in pages.iter().zip(&paths) {
            let mut lines = block.lines();
            let title = lines.next().unwrap();
            assert!(
                title.starts_with("# ") && title.len() > 2,
                "{path}: {title}"
            );
            assert_eq!(lines.next(), Some(""), "{path}");
            assert_eq!(lines.next(), Some(format!("Source: {SITE}{path}").as_str()));
            assert_eq!(lines.next(), Some(""), "{path}");
            assert!(
                lines.next().is_some_and(|l| !l.is_empty()),
                "{path}: a body"
            );
            // Text, not markup: nothing of the page's HTML is left.
            for tag in ["<main", "<div", "<p>", "<a ", "<span", "<nav", "&#"] {
                assert!(!block.contains(tag), "{path}: {tag}");
            }
            // The copy control and the shell are not the page. (The statement names the
            // control in its own text, as `_Copy this page as_`.)
            assert!(
                !block.contains("Copy this page as Markdown"),
                "{path}: the copy control"
            );
            assert!(
                !block.contains("[Source](https://github.com/bundu-labs/bundu-standards)"),
                "{path}: the header"
            );
            assert!(!block.contains("Published by the"), "{path}: the footer");
        }
    }

    #[test]
    fn every_page_is_listed_in_the_index_with_its_description() {
        assert!(INDEX.starts_with("# Bundu standards\n\n> "));
        assert!(INDEX.contains(&format!("{SITE}/llms-full.txt")));
        let mut titles = std::collections::BTreeSet::new();
        for (_, entries) in ENTRIES.iter() {
            for e in entries {
                assert!(titles.insert(&e.title), "two pages titled {}", e.title);
                let line = format!("- [{}]({}): {}\n", e.title, e.url, e.description);
                assert_eq!(INDEX.matches(&line).count(), 1, "{line}");
                assert!(!e.description.is_empty(), "{}", e.url);
            }
        }
        let n = REGISTER.standards.len() + DOCUMENTS.len() + 4;
        assert_eq!(INDEX.lines().filter(|l| l.starts_with("- [")).count(), n);
    }

    #[test]
    fn a_register_entry_reads_as_its_copy_control_copies_it() {
        let s = &REGISTER.standards[0];
        let path = format!("/register/{}/", s.id);
        let e = entry(&path);
        assert_eq!(e.title, format!("{} ({})", s.name, s.status));
        assert!(e.body.starts_with("**Governs**\n\n"), "{}", e.body);
        assert!(e
            .body
            .contains(&format!("**Specification**\n\n[{0}]({0})", s.url)));
        assert!(!e.body.contains("← Register"), "the breadcrumb is skipped");
        let h3 = entry("/register/h3/");
        assert!(h3.body.contains("`*_by_cell_*`"), "code stays code");
    }

    #[test]
    fn the_register_tab_and_the_statement_keep_their_structure() {
        let reg = entry("/register/");
        assert_eq!(reg.title, "Register");
        assert_eq!(entry("/register/proposed/").title, "Proposed register");
        assert!(
            !reg.body.contains("On this page"),
            "skipped, as on the page"
        );
        assert!(reg.body.contains("\n\n## "), "the domains are headings");
        let id = &REGISTER.standards[0].id;
        assert!(reg.body.contains(&format!("]({SITE}/register/{id}/) (")));
        let a11y = entry("/accessibility/");
        assert!(a11y.body.contains("## Using a keyboard"));
        assert!(a11y
            .body
            .contains("**Keys**\n\n| Key | What it does |\n| --- | --- |\n| `Tab` |"));
        let home = entry("/");
        assert!(home
            .body
            .contains("```\nGET /register/latest\nGET /<commit>/register/register.yaml\n```"));
        assert!(home.body.contains(&format!("[register]({SITE}/register/)")));
    }

    #[test]
    fn our_standards_read_as_their_readme_with_absolute_links() {
        let e = entry("/ai-guardrails/");
        assert_eq!(e.title, "Bundu AI Guardrails Standard (BAGS)");
        let readme = include_str!("../../../standards/ai-guardrails/README.md");
        assert!(
            readme.contains("](bags-1.0.0.json)"),
            "relative in the source"
        );
        assert!(!e.body.contains("](bags-1.0.0.json)"));
        assert!(e
            .body
            .contains(&format!("]({SITE}/ai-guardrails/bags-1.0.0.json)")));
        assert!(e.body.contains("\n## "), "headings kept");
        assert!(e.body.contains("> "), "quotes kept");
        assert!(e.body.contains(&format!(
            "## Files\n\n- [`bags-1.0.0.json`]({SITE}/ai-guardrails/bags-1.0.0.json)\n"
        )));
        assert!(!e.body.starts_with("# "), "the title is not repeated");
    }

    #[test]
    fn links_resolve_as_a_browser_resolves_them() {
        let base = "https://standards.bundu.org/conversation-interchange/";
        assert_eq!(
            resolve(base, "context.jsonld"),
            format!("{base}context.jsonld")
        );
        assert_eq!(
            resolve(base, "examples/minimal.ndjson"),
            format!("{base}examples/minimal.ndjson")
        );
        assert_eq!(resolve(base, "/register/"), format!("{SITE}/register/"));
        assert_eq!(resolve(base, "../cell/"), format!("{SITE}/cell/"));
        assert_eq!(resolve(base, "#x"), format!("{base}#x"));
        assert_eq!(resolve(base, "https://a.example/b"), "https://a.example/b");
        assert_eq!(
            resolve(base, "mailto:x@example.org"),
            "mailto:x@example.org"
        );
        assert_eq!(resolve(base, "//cdn.example/x"), "https://cdn.example/x");
        assert_eq!(
            absolutize("[a](a.json) [b](https://x.example/) `[c](c d)`", base),
            format!("[a]({base}a.json) [b](https://x.example/) `[c](c d)`")
        );
    }

    #[test]
    fn the_reader_decodes_references_and_skips_what_the_copy_control_skips() {
        let doc = html::parse(
            r#"<!doctype html><main><div data-md-skip="">x</div><p hidden="hidden">y</p><p>a &#60;b&#62; &amp; <kbd>Tab</kbd><br/>c</p><ul><li>one <span data-md=" (adopted)"><span class="sr-only">Status: </span>adopted</span><div>more</div></li></ul></main>"#,
        );
        let main = doc.find(&|e| e.tag == "main").unwrap();
        let mut out = Vec::new();
        markdown::blocks(&main.children, &mut out, SITE);
        assert_eq!(out, ["a <b> & `Tab`\nc", "- one (adopted) — more"]);
    }
}
