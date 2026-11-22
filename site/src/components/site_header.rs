//! `SiteHeader`: the brand and the main navigation.
//!
//! CONTRACT (for upstreaming):
//! - `<header class="site">` holding the brand link (the Bundu mark as a light and a dark image,
//!   picked by the colour scheme; both `alt=""` because the link's text names it) and one
//!   `<nav aria-label="Main">` with a list of links.
//! - The link to the current page carries `aria-current="page"`; a link to a section the page
//!   is inside carries `aria-current="location"`.
//! - A link whose text is short may carry a `hint`, read by a screen reader and not shown
//!   (`.sr-only`), so "JSON" is announced as "JSON (the register as JSON data)".
//! - Every link is a full hit region (`min-height: var(--hit)`: 45px on touch, 24px with a fine
//!   pointer) and the gaps between them are at least the 9px floor.

use dioxus::prelude::*;

/// One link of the main navigation.
#[derive(Clone, PartialEq, Debug)]
pub struct NavLink {
    pub label: String,
    pub href: String,
    pub hint: Option<String>,
}

impl NavLink {
    pub fn new(label: &str, href: &str) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            hint: None,
        }
    }

    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// `aria-current` for a link on the page at `path`: `page` for the page itself, `location`
/// for a page inside the section the link leads to. Links to other sites have none.
pub fn current(path: &str, href: &str) -> Option<&'static str> {
    if !href.starts_with('/') {
        return None;
    }
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
pub fn SiteHeader(path: String, brand: String, links: Vec<NavLink>) -> Element {
    rsx! {
        header { class: "site",
            a { class: "brand", href: "/", "aria-current": current(&path, "/"),
                img { class: "mark-light", src: "/favicon-32x32.png", alt: "", width: "24", height: "24" }
                img { class: "mark-dark", src: "/favicon-32x32-dark.png", alt: "", width: "24", height: "24" }
                "{brand}"
            }
            nav { "aria-label": "Main",
                ul {
                    for link in links {
                        li {
                            a { href: "{link.href}", "aria-current": current(&path, &link.href),
                                "{link.label}"
                                if let Some(hint) = link.hint {
                                    span { class: "sr-only", " ({hint})" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
