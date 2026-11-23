//! standards.bundu.org as a small native Rust server: the pages and the file endpoint.
//!
//! The behaviour lives in [`core`] (the file endpoint), [`site`] and [`pages`] (the pages,
//! rendered with Dioxus and Mzizi Roots), [`llms`] (the pages as text, for machines) and
//! [`assets`] (the static files and their headers),
//! none of which touches the network; all are tested natively (`cargo test`). [`server`] is
//! the dispatch between them and a cache of the file endpoint's answers, and [`github`] is the
//! HTTPS client the file endpoint reads GitHub with. `src/main.rs` is only the listener.

pub mod assets;
pub mod core;
pub mod github;
pub mod llms;
pub mod openapi;
pub mod pages;
pub mod register;
pub mod server;
pub mod site;
