//! Authentication routes.
//!
//! These are the endpoints the frontend (mukoko.com, nyuchi product
//! consumer interfaces, native apps) calls during the login flow.
//!
//! Flow:
//!   1. Frontend → GET /auth/signin?redirect_uri=...
//!      → returns the AuthKit URL the user is redirected to.
//!   2. User authenticates on AuthKit hosted UI.
//!   3. WorkOS redirects user back to frontend with `?code=...`.
//!   4. Frontend → POST /auth/exchange { code } → receives platform JWT.
//!   5. Frontend uses platform JWT for subsequent calls to other services.

use std::sync::Arc;

use axum::{extract::{Query, State}, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppResult, AppState};

/// Build the auth router.
#[must_use]
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/signin", get(signin))
        .route("/auth/exchange", post(exchange))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
}

#[derive(Debug, Deserialize)]
struct SigninQuery {
    /// Optional redirect URI override (must be on the WorkOS allowlist).
    redirect_uri: Option<String>,
    /// Opaque CSRF/return state.
    state: Option<String>,
    /// Pre-fill email.
    login_hint: Option<String>,
}

#[derive(Debug, Serialize)]
struct SigninResponse {
    /// The AuthKit URL the frontend redirects the user to.
    authorization_url: String,
}

async fn signin(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SigninQuery>,
) -> AppResult<Json<SigninResponse>> {
    let redirect_uri = query
        .redirect_uri
        .as_deref()
        .unwrap_or(&state.config.workos.redirect_uri);
    let url = state.workos.authorization_url(
        redirect_uri,
        query.state.as_deref(),
        query.login_hint.as_deref(),
    )?;
    Ok(Json(SigninResponse {
        authorization_url: url.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
struct ExchangeRequest {
    code: String,
}

#[derive(Debug, Serialize)]
struct ExchangeResponse {
    /// The platform JWT the frontend stores and presents on subsequent
    /// requests.
    platform_jwt: String,
    /// Convenience: when the JWT expires (Unix seconds).
    expires_at: i64,
    /// Convenience: the active entity id.
    active_entity_id: uuid::Uuid,
}

async fn exchange(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<ExchangeRequest>,
) -> AppResult<Json<ExchangeResponse>> {
    // TODO once WorkOS is provisioned and the resolve_or_create_identity
    // SQL function is in place:
    //
    //   1. let session = state.workos.authenticate_with_code(&req.code).await?;
    //   2. let resolved = state.identity.resolve_or_create(
    //        &session.user.id, &session.user.email, ...
    //      ).await?;
    //   3. let platform_session = build_session(resolved, ...);
    //   4. let token = state.jwt.mint(&platform_session)?;
    //   5. state.bus.publish(&events::person_created(...)).await?;  // if newly created
    //   6. return Json(ExchangeResponse { platform_jwt: token, ... });
    //
    // Stubbed until WorkOS provisioning + SQL function are complete.
    Err(AppError::BadRequest(
        "exchange not yet implemented — WorkOS provisioning pending".into(),
    ))
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<RefreshRequest>,
) -> AppResult<Json<ExchangeResponse>> {
    Err(AppError::BadRequest("refresh not yet implemented".into()))
}

#[derive(Debug, Deserialize)]
struct LogoutRequest {
    session_id: String,
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LogoutRequest>,
) -> AppResult<impl IntoResponse> {
    state.workos.revoke_session(&req.session_id).await?;
    Ok(Json(serde_json::json!({ "status": "logged_out" })))
}
