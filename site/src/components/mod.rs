//! The components that have no Rust counterpart in the Mzizi registry yet, each in a module of
//! its own with its contract, so each can go upstream unchanged (see `site/README.md`).
//! The status badge, the one primitive the pages use, is `mzizi-roots`' `Badge`.

pub mod copy_control;
pub mod site_header;
pub mod skip_link;
