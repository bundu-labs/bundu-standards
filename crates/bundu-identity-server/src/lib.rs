//! Bundu Identity gateway server.
//!
//! Binds together:
//! - [`bundu_workos`] for WorkOS REST calls
//! - [`bundu_supabase`] for identity resolution
//! - [`bundu_jwt`] for platform JWT minting/validation
//! - [`bundu_service_bus`] for event publication
//!
//! Exposes the HTTP endpoints documented in the repository README.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

pub mod config;
pub mod error;
pub mod routes;
pub mod state;

pub use config::AppConfig;
pub use error::{AppError, AppResult};
pub use state::AppState;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

/// Build the axum [`Router`] for the gateway.
#[must_use]
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::auth::router())
        .merge(routes::entity::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
