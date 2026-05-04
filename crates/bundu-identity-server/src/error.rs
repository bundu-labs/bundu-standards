//! Application error type and RFC 7807 HTTP response mapping.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// All errors that can flow up to a route handler.
#[derive(Debug, Error)]
pub enum AppError {
    /// WorkOS-side failure.
    #[error("workos error: {0}")]
    WorkOs(#[from] bundu_workos::WorkOsError),

    /// Supabase-side failure.
    #[error("supabase error: {0}")]
    Supabase(#[from] bundu_supabase::SupabaseError),

    /// JWT minting/validation failure.
    #[error("jwt error: {0}")]
    Jwt(#[from] bundu_jwt::JwtError),

    /// Service bus publish failure.
    #[error("service bus error: {0}")]
    Bus(#[from] bundu_service_bus::ServiceBusError),

    /// Bad request from client.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Not authenticated.
    #[error("unauthorized")]
    Unauthorized,

    /// Forbidden — authenticated but lacks permission.
    #[error("forbidden")]
    Forbidden,

    /// Catch-all internal error.
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Convenience alias.
pub type AppResult<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, type_uri, title) = match &self {
            Self::WorkOs(bundu_workos::WorkOsError::SessionInvalid(_))
            | Self::WorkOs(bundu_workos::WorkOsError::NotFound(_)) => {
                (StatusCode::UNAUTHORIZED, "https://id.mukoko.com/errors/auth-session-invalid", "Authentication failed")
            }
            Self::BadRequest(_) => {
                (StatusCode::BAD_REQUEST, "https://id.mukoko.com/errors/bad-request", "Bad request")
            }
            Self::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "https://id.mukoko.com/errors/unauthorized", "Unauthorized")
            }
            Self::Forbidden => {
                (StatusCode::FORBIDDEN, "https://id.mukoko.com/errors/forbidden", "Forbidden")
            }
            _ => {
                tracing::error!(error = ?self, "internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "https://id.mukoko.com/errors/internal", "Internal server error")
            }
        };

        let body = Json(json!({
            "type": type_uri,
            "title": title,
            "status": status.as_u16(),
            "detail": self.to_string(),
        }));

        (status, body).into_response()
    }
}
