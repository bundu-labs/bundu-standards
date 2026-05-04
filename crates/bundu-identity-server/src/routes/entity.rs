//! Entity context routes — listing the user's entities and switching
//! the active one in the platform JWT.

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, AppResult, AppState};

/// Build the entity router.
#[must_use]
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/entity/list", get(list_entities))
        .route("/entity/switch", post(switch_entity))
}

async fn list_entities(State(_state): State<Arc<AppState>>) -> AppResult<impl IntoResponse> {
    // TODO: extract person_id from validated JWT (auth middleware), then
    // list entities via state.identity.list_entities_for_person(person_id).
    Err::<Json<Vec<()>>, _>(AppError::BadRequest("list_entities not yet implemented".into()))
}

#[derive(Debug, Deserialize)]
struct SwitchRequest {
    entity_id: Uuid,
}

async fn switch_entity(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<SwitchRequest>,
) -> AppResult<impl IntoResponse> {
    // TODO: validate caller's JWT, verify membership in target entity,
    // re-mint JWT with new active_context. Returns new JWT.
    Err::<Json<()>, _>(AppError::BadRequest("switch_entity not yet implemented".into()))
}
