//! Health and readiness endpoints.

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::AppState;

/// Build the health router.
#[must_use]
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(liveness))
        .route("/health/ready", get(readiness))
}

async fn liveness() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn readiness(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // TODO: actually probe Supabase + WorkOS reachability.
    // For scaffold: always-ready, replaced when concrete clients implemented.
    Json(json!({ "status": "ready" }))
}
