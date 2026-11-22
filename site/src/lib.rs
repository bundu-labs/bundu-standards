//! The page shell, stylesheet and accessibility audit that standards.bundu.org and
//! vocab.bundu.org share, so the two sites look and behave the same.
//!
//! [`Document`] is the whole page: head, skip link, header, `main` with the copy control, and
//! footer. The stylesheet is [`CSS`], served by each site at [`CSS_PATH`]. [`audit`] checks a
//! rendered page against the accessibility rules both sites keep, and [`css_audit`] checks the
//! stylesheet itself; each site runs both over its pages in `cargo test`.

pub mod audit;
pub mod components;
pub mod css_audit;

use components::copy_control::CopyControl;
use components::site_header::SiteHeader;
use components::skip_link::SkipLink;
use dioxus::prelude::*;

pub use components::site_header::{current, NavLink};

/// The stylesheet both sites serve.
pub const CSS: &str = include_str!("../styles/site.css");
pub const CSS_PATH: &str = "/styles/site.css";

/// What a site tells the shell about a page.
#[derive(Clone, PartialEq)]
pub struct Page {
    /// The path, for `aria-current` and the canonical link.
    pub path: String,
    /// Whether the page has a canonical URL. A page without one (an error page) is `noindex`.
    pub canonical: bool,
    /// The site's origin, no trailing slash (`https://standards.bundu.org`).
    pub origin: String,
    pub title: String,
    pub description: String,
    /// The brand beside the mark.
    pub brand: String,
    pub links: Vec<NavLink>,
    /// "JSON" or "JSON-LD", for the copy control.
    pub json_label: String,
    /// Structured data for the page, as a JSON string with `<` already escaped.
    pub json_ld: Option<String>,
    /// The data the copy control offers as JSON, with its script type.
    pub page_json: Option<(String, String)>,
}

#[component]
pub fn Document(
    page: Page,
    /// More `<head>` elements for the site (a `service-desc` link, say).
    head: Element,
    footer: Element,
    children: Element,
) -> Element {
    let canonical = format!("{}{}", page.origin, page.path);
    rsx! {
        head {
            meta { charset: "utf-8" }
            meta { name: "viewport", content: "width=device-width, initial-scale=1" }
            title { "{page.title}" }
            meta { name: "description", content: "{page.description}" }
            if page.canonical {
                link { rel: "canonical", href: "{canonical}" }
            } else {
                meta { name: "robots", content: "noindex" }
            }
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
            {head}
            if let Some(ld) = page.json_ld {
                script { r#type: "application/ld+json", dangerous_inner_html: "{ld}" }
            }
            // The data the copy control offers as JSON; pages without it offer Markdown only.
            if let Some((kind, data)) = page.page_json {
                script { r#type: "{kind}", id: "page-json", dangerous_inner_html: "{data}" }
            }
            script { src: "/copy.js", defer: "defer" }
        }
        body {
            SkipLink {}
            SiteHeader { path: page.path.clone(), brand: page.brand.clone(), links: page.links.clone() }
            main { id: "main", tabindex: "-1",
                CopyControl { json_label: page.json_label.clone() }
                {children}
            }
            footer { class: "site", {footer} }
        }
    }
}

/// Render a page to a whole HTML document: `<!doctype html>`, the `html` element (Dioxus has
/// none, so it is written here) and the shell around `main`.
pub fn render(page: Page, head: Element, footer: Element, main: Element) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        Document { page, head, footer, {main} }
    });
    format!("<!doctype html><html lang=\"en\">{body}</html>")
}

/// [`render`] for a site whose footer and page body are HTML it already has (the vocabulary's
/// pages are written as HTML). Both go in as they are, unescaped: the caller owns their safety.
pub fn render_html(page: Page, head_html: &str, footer_html: &str, body_html: &str) -> String {
    const BODY: &str = "BUNDUSITEBODYMARKER";
    const FOOTER: &str = "BUNDUSITEFOOTERMARKER";
    const HEAD: &str = "BUNDUSITEHEADMARKER";
    let html = render(
        page,
        rsx! { meta { name: HEAD } },
        rsx! { "{FOOTER}" },
        rsx! { "{BODY}" },
    );
    html.replacen(&format!("<meta name=\"{HEAD}\"/>"), head_html, 1)
        .replacen(FOOTER, footer_html, 1)
        .replacen(BODY, body_html, 1)
}
