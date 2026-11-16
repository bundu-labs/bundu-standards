//! The pages of standards.bundu.org, rendered on the server with Dioxus (Mzizi Roots).
//!
//! The Rust port of `workers/standards/src/layouts/Base.astro` and its four pages. Same
//! markup, same classes, same text: `site.css` styles both. A standard's status is the Mzizi
//! [`Badge`] (`mzizi-ui`), which carries the registry's `data-slot` and `data-portal`; the
//! site ships no Tailwind, so its `.chip` rules give the badge its look.

use crate::register::{self, Register, Standard, SITE};
use dioxus::prelude::*;
use mzizi_ui::{Badge, BadgeVariant};
use std::sync::LazyLock;

/// The register, parsed once per isolate from the JSON `build.rs` wrote.
pub static REGISTER: LazyLock<Register> = LazyLock::new(|| {
    Register::parse(include_str!(concat!(env!("OUT_DIR"), "/register.json")))
        .expect("checked by build.rs")
});

/// The stylesheet, served at [`CSS_PATH`]. One file for the Astro and the Rust builds.
pub const CSS: &str = include_str!("../../standards/src/styles/site.css");
pub const CSS_PATH: &str = "/styles/site.css";

/// A rendered page or document.
pub struct Page {
    pub content_type: &'static str,
    pub body: String,
}

/// The page for `path`, or `None` when this module has no page there. Trailing slashes are
/// optional, as with Astro's `trailingSlash: "ignore"`.
pub fn render(path: &str) -> Option<Page> {
    let trimmed = path.trim_end_matches('/');
    let r = &*REGISTER;
    let html = |body: String| {
        Some(Page {
            content_type: "text/html; charset=utf-8",
            body,
        })
    };
    match trimmed {
        "" => html(document(path, rsx! { Index {} }, home_meta(), None)),
        "/register" => html(document(
            path,
            rsx! { RegisterIndex {} },
            (
                "Register · Bundu standards".into(),
                "Every external standard the platform adopts, and its status.".into(),
            ),
            Some(register::defined_term_set(r, SITE)),
        )),
        "/register/index.json" => Some(Page {
            content_type: "application/json; charset=utf-8",
            body: register::index_json(r, SITE),
        }),
        "/accessibility" => html(document(
            path,
            rsx! { Accessibility {} },
            accessibility_meta(),
            None,
        )),
        _ => {
            let id = trimmed.strip_prefix("/register/")?;
            let s = r.get(id)?;
            html(document(
                path,
                rsx! { StandardPage { id: s.id.clone() } },
                (
                    format!("{} · Bundu standards", s.name),
                    register::one_line(&s.governs),
                ),
                Some(register::defined_term_page(s, SITE)),
            ))
        }
    }
}

fn home_meta() -> (String, String) {
    (
        "Bundu standards".into(),
        "The open standards the Bundu, Mukoko and Nyuchi platform builds on.".into(),
    )
}

fn accessibility_meta() -> (String, String) {
    (
        "Accessibility · Bundu standards".into(),
        "How standards.bundu.org and vocab.bundu.org work for everyone, with a keyboard, a screen reader, zoom or colour-blind vision, and how to report a barrier.".into(),
    )
}

/// The whole document: `<!doctype html>` and the [`Base`] layout around `main`.
fn document(
    path: &str,
    main: Element,
    (title, description): (String, String),
    json_ld: Option<serde_json::Value>,
) -> String {
    // `</script>` cannot appear inside the JSON: every `<` is written as <.
    let json_ld = json_ld.map(|v| v.to_string().replace('<', "\\u003c"));
    let body = dioxus_ssr::render_element(rsx! {
        Base { path: path.to_owned(), title, description, json_ld, {main} }
    });
    // Dioxus has no `html` element, so the root is written here.
    format!("<!doctype html><html lang=\"en\">{body}</html>")
}

/// aria-current for a main navigation link, as Base.astro worked it out: the register's
/// pages count as the register.
fn current(path: &str, href: &str) -> Option<&'static str> {
    if href == "/" {
        return (path == "/").then_some("page");
    }
    if path == href {
        Some("page")
    } else if path.starts_with(href) {
        Some("location")
    } else {
        None
    }
}

#[component]
fn Base(
    path: String,
    title: String,
    description: String,
    json_ld: Option<String>,
    children: Element,
) -> Element {
    let canonical = format!("{SITE}{path}");
    rsx! {
        head {
                meta { charset: "utf-8" }
                meta { name: "viewport", content: "width=device-width, initial-scale=1" }
                title { "{title}" }
                meta { name: "description", content: "{description}" }
                link { rel: "canonical", href: "{canonical}" }
                // The official Bundu mark (copper), as shipped by www.bundu.org: a light-surface
                // and a dark-surface pair, picked by the browser's colour scheme. Never redrawn.
                link { rel: "icon", r#type: "image/png", sizes: "32x32", href: "/favicon-32x32.png", media: "(prefers-color-scheme: light)" }
                link { rel: "icon", r#type: "image/png", sizes: "16x16", href: "/favicon-16x16.png", media: "(prefers-color-scheme: light)" }
                link { rel: "icon", r#type: "image/png", sizes: "32x32", href: "/favicon-32x32-dark.png", media: "(prefers-color-scheme: dark)" }
                link { rel: "icon", r#type: "image/png", sizes: "16x16", href: "/favicon-16x16-dark.png", media: "(prefers-color-scheme: dark)" }
                link { rel: "apple-touch-icon", sizes: "180x180", href: "/apple-touch-icon.png" }
                meta { name: "theme-color", content: "#fbfaf7", "media": "(prefers-color-scheme: light)" }
                meta { name: "theme-color", content: "#141412", "media": "(prefers-color-scheme: dark)" }
                link { rel: "stylesheet", href: CSS_PATH }
                if let Some(ld) = json_ld {
                    script { r#type: "application/ld+json", dangerous_inner_html: "{ld}" }
                }
            }
            body {
                a { class: "skip", href: "#main", "Skip to main content" }
                header { class: "site",
                    a { class: "brand", href: "/", "aria-current": current(&path, "/"),
                        img { class: "mark-light", src: "/favicon-32x32.png", alt: "", width: "24", height: "24" }
                        img { class: "mark-dark", src: "/favicon-32x32-dark.png", alt: "", width: "24", height: "24" }
                        "Bundu standards"
                    }
                    nav { "aria-label": "Main",
                        ul {
                            li { a { href: "/register/", "aria-current": current(&path, "/register/"), "Register" } }
                            li { a { href: "/register/index.json", "JSON", span { class: "sr-only", " (the register as JSON data)" } } }
                            li { a { href: "https://github.com/bundu-labs/bundu-standards", "Source", span { class: "sr-only", " code on GitHub" } } }
                            li { a { href: "/accessibility/", "aria-current": current(&path, "/accessibility/"), "Accessibility" } }
                        }
                    }
                }
                main { id: "main", tabindex: "-1", {children} }
                footer { class: "site",
                    p {
                        "Published by the "
                        a { href: "https://www.bundu.org", "Bundu Foundation" }
                        ". The register is CC0."
                    }
                    p {
                        "Every page works with a keyboard alone: "
                        kbd { "Tab" }
                        " moves forward, "
                        kbd { "Shift" }
                        "+"
                        kbd { "Tab" }
                        " moves back, "
                        kbd { "Enter" }
                        " follows a link. Read the "
                        a { href: "/accessibility/", "accessibility statement" }
                        " or "
                        a { href: "https://github.com/bundu-labs/bundu-standards/issues/new?title=Accessibility%3A%20", "report a barrier" }
                        "."
                    }
                }
            }
    }
}

/// A standard's status: the Mzizi badge, read as "Status: adopted" by a screen reader.
#[component]
fn Status(status: String) -> Element {
    let variant = if status == "adopted" {
        BadgeVariant::Default
    } else {
        BadgeVariant::Secondary
    };
    rsx! {
        Badge { variant, class: "chip {status}",
            span { class: "sr-only", "Status: " }
            "{status}"
        }
    }
}

#[component]
fn Index() -> Element {
    let c = REGISTER.counts;
    rsx! {
        h1 { "Bundu standards" }
        p {
            "If a standard exists we adopt it. If none exists we build one and publish it. Data from a source that does not match is converted at the edge, and only the standard form is stored or served."
        }
        p {
            "The "
            a { href: "/register/", "register" }
            " lists {c.total} standards: {c.adopted} adopted, {c.proposed} proposed."
        }
        h2 { "For other repositories" }
        p { "Read the register at a commit, with no credential. Pin the sha256 of each file." }
        pre { tabindex: "0", role: "region", "aria-label": "Example requests",
            "GET /register/latest\nGET /<commit>/register/register.yaml"
        }
    }
}

#[component]
fn RegisterIndex() -> Element {
    let r = &*REGISTER;
    rsx! {
        h1 { "Register" }
        p { "{r.counts.adopted} adopted, {r.counts.proposed} proposed." }
        nav { "aria-labelledby": "on-this-page",
            h2 { id: "on-this-page", "On this page" }
            ul { class: "toc",
                for d in r.domains.iter() {
                    li { a { href: "#{d.key}", "{d.title}" } }
                }
            }
        }
        for d in r.domains.iter() {
            section { "aria-labelledby": "{d.key}",
                h2 { id: "{d.key}", tabindex: "-1", "{d.title}" }
                ul { class: "list",
                    for s in d.standards.iter() {
                        li {
                            a { href: "/register/{s.id}/", "{s.name}" }
                            Status { status: s.status.clone() }
                            div { class: "muted", "{s.current}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StandardPage(id: String) -> Element {
    let Some(s): Option<&Standard> = REGISTER.get(&id) else {
        return rsx! {};
    };
    rsx! {
        nav { "aria-label": "Breadcrumb",
            p { a { href: "/register/", span { "aria-hidden": "true", "← " } "Register" } }
        }
        h1 { "{s.name} " Status { status: s.status.clone() } }
        dl {
            dt { "Governs" } dd { "{s.governs}" }
            dt { "Current" } dd { "{s.current}" }
            dt { "Commitment" } dd { "{s.commitment}" }
            if let Some(steward) = &s.steward {
                dt { "Steward" } dd { "{steward}" }
            }
            dt { "Decided" } dd { "{s.decided}" }
            dt { "Specification" } dd { a { href: "{s.url}", "{s.url}" } }
            if let Some(docs) = &s.docs {
                dt { "Documentation" } dd { a { href: "{docs}", "{docs}" } }
            }
            if let Some(notes) = &s.notes {
                dt { "Notes" } dd { "{notes}" }
            }
        }
    }
}

#[component]
fn Accessibility() -> Element {
    rsx! {
        h1 { "Accessibility" }
        p {
            "The standards register and the Bundu vocabulary are public infrastructure, so everyone has to be able to use them. Both sites are built to the Web Content Accessibility Guidelines (WCAG) 2.2 at level AA, and to the Bundu floor, which goes further: body text at 7:1 contrast, text no smaller than 14px, and controls large enough to hit."
        }
        h2 { id: "keyboard", "Using a keyboard" }
        p { "Every page works with a keyboard alone. Nothing needs a mouse, and nothing traps the focus." }
        table {
            caption { "Keys" }
            thead {
                tr { th { scope: "col", "Key" } th { scope: "col", "What it does" } }
            }
            tbody {
                tr { td { kbd { "Tab" } } td { "Moves to the next link or control. The first stop on every page is " em { "Skip to main content" } "." } }
                tr { td { kbd { "Shift" } " + " kbd { "Tab" } } td { "Moves back to the previous link or control." } }
                tr { td { kbd { "Enter" } } td { "Follows the focused link." } }
                tr { td { kbd { "Space" } " or " kbd { "Page Down" } } td { "Scrolls the page down; " kbd { "Shift" } " + " kbd { "Space" } " or " kbd { "Page Up" } " scrolls up." } }
                tr { td { "Arrow keys" } td { "Scroll an example block sideways once it has the focus." } }
            }
        }
        p {
            "The focused item always shows a visible outline in copper. On the register, "
            em { "On this page" }
            " jumps straight to each group of standards."
        }
        h2 { id: "screen-readers", "Using a screen reader" }
        ul {
            li { "Each page has one main region, a labelled main navigation and a footer, so landmark navigation works." }
            li { "Headings are in order: one level-one heading per page, then a level-two heading for each section." }
            li { "The page you are on is announced as the current page in the navigation." }
            li { "A standard's status is read as “Status: adopted” or “Status: proposed”, not as a colour." }
            li { "Links say where they go, including the ones whose visible text is short, such as “JSON” and “Source”." }
            li { "Images that only decorate, such as the Bundu mark beside the site name, are hidden from screen readers." }
        }
        h2 { id: "vision", "Zoom, contrast and colour" }
        ul {
            li { "Text can be zoomed to 400% and the page reflows to one column with no sideways scrolling." }
            li { "Light and dark themes follow your device setting. Both meet the contrast floor." }
            li { "Colour never carries meaning on its own: every status and every colour-scale band also has a text label." }
            li { "The weather colour scales publish, for each band, where its label goes and which surfaces it needs an outline on, and the scales we designed stay distinguishable for people with red, green or blue colour blindness." }
        }
        h2 { id: "data", "Data without the pages" }
        p {
            "Everything on these pages is also available as data: the register as "
            a { href: "/register/index.json", "JSON" }
            " and each vocabulary term as JSON-LD. Assistive tools and scripts can read it directly."
        }
        h2 { id: "checked", "How this is checked" }
        p {
            "Every change runs the site's tests, which fail if the skip link, the labelled navigation, the main region or a text label goes missing. The pages are also audited with axe-core against WCAG 2.2 AA in the light and dark themes; the last audit, on 15 November 2026, found no violations."
        }
        h2 { id: "report", "Report a barrier" }
        p {
            "If something on these sites does not work for you, tell us. Open an issue at "
            a { href: "https://github.com/bundu-labs/bundu-standards/issues/new?title=Accessibility%3A%20", "github.com/bundu-labs/bundu-standards" }
            ", saying which page, what you were trying to do and what you use (for example a screen reader or zoom). A barrier is treated as a bug, not a feature request."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(path: &str) -> String {
        render(path)
            .unwrap_or_else(|| panic!("no page at {path}"))
            .body
    }

    #[test]
    fn every_page_keeps_the_accessibility_landmarks() {
        let mut paths = vec![
            "/".to_owned(),
            "/register/".into(),
            "/accessibility/".into(),
        ];
        paths.extend(
            REGISTER
                .standards
                .iter()
                .map(|s| format!("/register/{}/", s.id)),
        );
        for p in paths {
            let html = page(&p);
            assert!(html.starts_with("<!doctype html><html lang=\"en\">"), "{p}");
            assert!(
                html.contains(r##"<a class="skip" href="#main">Skip to main content</a>"##),
                "{p}"
            );
            assert!(html.contains(r#"<nav aria-label="Main">"#), "{p}");
            assert!(html.contains(r#"<main id="main" tabindex="-1">"#), "{p}");
            assert_eq!(html.matches("<h1").count(), 1, "{p}: one h1");
            assert!(
                html.contains(&format!(r#"<link rel="canonical" href="{SITE}{p}"/>"#)),
                "{p}"
            );
        }
    }

    #[test]
    fn the_current_page_is_marked_in_the_navigation() {
        assert!(page("/").contains(r#"<a class="brand" href="/" aria-current="page">"#));
        assert!(page("/register/").contains(r#"<a href="/register/" aria-current="page">"#));
        let id = &REGISTER.standards[0].id;
        assert!(page(&format!("/register/{id}/"))
            .contains(r#"<a href="/register/" aria-current="location">"#));
        assert!(!page("/accessibility/").contains(r#"href="/register/" aria-current"#));
    }

    #[test]
    fn a_status_is_a_mzizi_badge_with_a_text_label() {
        let html = page("/register/");
        assert!(html.contains(r#"data-slot="badge""#));
        assert!(html.contains(r#"<span class="sr-only">Status: </span>adopted</span>"#));
        let c = REGISTER.counts;
        assert_eq!(html.matches(r#"data-slot="badge""#).count(), c.total);
        assert!(html.contains(&format!(
            "<p>{} adopted, {} proposed.</p>",
            c.adopted, c.proposed
        )));
    }

    #[test]
    fn text_is_escaped_and_json_ld_cannot_close_its_script() {
        let home = page("/");
        assert!(home.contains("GET /&#60;commit&#62;/register/register.yaml"));
        let reg = page("/register/");
        let start = reg.find(r#"<script type="application/ld+json">"#).unwrap();
        let ld = &reg[start..reg[start..].find("</script>").unwrap() + start];
        assert!(!ld[1..].contains('<'));
        assert!(ld.contains(r#""@type":"DefinedTermSet""#));
    }

    #[test]
    fn unknown_paths_have_no_page() {
        assert!(render("/register/not-a-standard/").is_none());
        assert!(render("/nope").is_none());
        assert!(render("/register/latest").is_none());
    }

    #[test]
    fn the_index_json_lists_every_standard() {
        let p = render("/register/index.json").unwrap();
        assert_eq!(p.content_type, "application/json; charset=utf-8");
        let v: serde_json::Value = serde_json::from_str(&p.body).unwrap();
        assert_eq!(
            v["standards"].as_array().unwrap().len(),
            REGISTER.counts.total
        );
    }
}
