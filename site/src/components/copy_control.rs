//! `CopyControl`: "Copy this page as" Markdown or JSON, driven by `/copy.js`.
//!
//! CONTRACT (for upstreaming):
//! - A `div.copy[data-copy][hidden]`: hidden until the script runs and unhides it, so no reader
//!   meets a button that does nothing.
//! - A labelled `select` (`label for="copy-format"`), a `button type="button"` and a
//!   `span[role=status]` that announces the result.
//! - The formats are Markdown and the page's data (`json_label`: "JSON" or "JSON-LD"); never HTML.
//! - The select and the button are full hit regions (`min-height: var(--hit)`).
//! - `data-md-skip` keeps the control itself out of the Markdown it produces.

use dioxus::prelude::*;

#[component]
pub fn CopyControl(json_label: String) -> Element {
    rsx! {
        div { class: "copy", "data-copy": "", "data-md-skip": "", hidden: "hidden",
            label { r#for: "copy-format", "Copy this page as" }
            select { id: "copy-format",
                option { value: "markdown", "Markdown" }
                option { value: "json", "{json_label}" }
            }
            button { r#type: "button", "Copy" }
            span { class: "copy-status", role: "status" }
        }
    }
}
