//! `SkipLink`: the first focusable element of every page, a link to `#main`.
//!
//! CONTRACT (for upstreaming to mzizi-registry as a Rust N3 component):
//! - It is the first element of `<body>`, before the header, so a keyboard user reaches it with
//!   the first Tab (WCAG 2.4.1 Bypass Blocks).
//! - It is hidden off-screen until it takes focus, then shown, with a 2px accent border.
//! - It targets `#main`, which the shell gives `tabindex="-1"` so focus lands on it.
//! - Text: "Skip to main content". Nothing else is configurable.

use dioxus::prelude::*;

#[component]
pub fn SkipLink() -> Element {
    rsx! { a { class: "skip", href: "#main", "Skip to main content" } }
}
