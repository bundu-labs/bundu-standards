//! WorkOS REST client.
//!
//! This crate is the **sole** integration boundary between the Bundu
//! ecosystem and WorkOS. Every other crate consumes the [`WorkOsClient`]
//! trait — concrete HTTP details live here. This makes the gateway
//! testable (via the [`MockWorkOsClient`] in tests) and migratable
//! (when Phase 3+ self-hosts auth, only this crate is rewritten).
//!
//! The mapping (locked in v4.0.3):
//! - WorkOS User           ↔ `identity.person`
//! - WorkOS Organization   ↔ `identity.entity` (`entity_type='organization'`)
//! - WorkOS OrgMembership  ↔ `entity.membership`
//!
//! Family entities (per v4.0.3) are NOT mirrored to WorkOS — they exist
//! purely on the platform side. Only verification-gated organisations
//! get a WorkOS Organization counterpart.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Errors returned by the WorkOS client.
#[derive(Debug, Error)]
pub enum WorkOsError {
    /// Configuration is missing or invalid.
    #[error("workos configuration error: {0}")]
    Config(String),
    /// Authorization code or session token is invalid.
    #[error("workos session invalid: {0}")]
    SessionInvalid(String),
    /// WorkOS user/org/membership not found.
    #[error("workos resource not found: {0}")]
    NotFound(String),
    /// Underlying HTTP error.
    #[error("workos http error: {0}")]
    Http(#[from] reqwest::Error),
    /// Url parse error.
    #[error("workos url parse error: {0}")]
    Url(#[from] url::ParseError),
    /// JSON parse error.
    #[error("workos json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Catch-all for unexpected WorkOS responses.
    #[error("unexpected workos response: {0}")]
    Unexpected(String),
}

/// Result alias.
pub type WorkOsResult<T> = std::result::Result<T, WorkOsError>;

/// A WorkOS user as projected into the gateway's domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOsUser {
    /// WorkOS user id (e.g. `user_01HXYZ...`).
    pub id: String,
    /// Verified email address.
    pub email: String,
    /// Whether WorkOS has verified the email.
    pub email_verified: bool,
    /// Given name, if WorkOS has it.
    pub first_name: Option<String>,
    /// Family name, if WorkOS has it.
    pub last_name: Option<String>,
    /// Profile picture URL, if any.
    pub profile_picture_url: Option<String>,
}

/// A WorkOS organisation as projected into the gateway's domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOsOrganization {
    /// WorkOS org id (e.g. `org_01HXYZ...`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Verified domains attached to this org.
    pub domains: Vec<String>,
}

/// A WorkOS organisation membership.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOsMembership {
    /// Membership id.
    pub id: String,
    /// Linked user id.
    pub user_id: String,
    /// Linked organisation id.
    pub organization_id: String,
    /// Role slug (`admin`, `member`, etc.).
    pub role_slug: String,
}

/// The result of a successful WorkOS authentication exchange.
#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    /// The authenticated user.
    pub user: WorkOsUser,
    /// All the user's active memberships at exchange time.
    pub memberships: Vec<WorkOsMembership>,
    /// All organisations referenced by `memberships` (parallel order).
    pub organizations: Vec<WorkOsOrganization>,
    /// WorkOS session id (for revocation).
    pub workos_session_id: String,
    /// Short-lived access token.
    pub workos_access_token: String,
    /// Refresh token, if WorkOS issued one.
    pub workos_refresh_token: Option<String>,
}

/// The behaviour the gateway needs from WorkOS. Implementations:
///
/// - [`HttpWorkOsClient`] — production, talks to api.workos.com.
/// - `MockWorkOsClient` (test) — in-memory fixture.
///
/// All operations are async. Implementations should be cheap to clone
/// (typically wrap the HTTP client in `Arc`).
#[async_trait]
pub trait WorkOsClient: Send + Sync {
    /// Build the AuthKit hosted-UI URL the frontend redirects to.
    fn authorization_url(
        &self,
        redirect_uri: &str,
        state: Option<&str>,
        login_hint: Option<&str>,
    ) -> WorkOsResult<Url>;

    /// Exchange an OAuth authorization code for a full session.
    async fn authenticate_with_code(&self, code: &str) -> WorkOsResult<AuthenticatedSession>;

    /// Refresh an expired session using its refresh token.
    async fn refresh(&self, refresh_token: &str) -> WorkOsResult<AuthenticatedSession>;

    /// Revoke a WorkOS session server-side.
    async fn revoke_session(&self, session_id: &str) -> WorkOsResult<()>;

    /// Build the WorkOS hosted-UI logout URL.
    fn logout_url(&self, session_id: &str) -> WorkOsResult<Url>;
}

/// HTTP implementation of [`WorkOsClient`].
///
/// Construct once at app startup; clone freely (the inner state is
/// `Arc`-shared inside `reqwest::Client`).
#[derive(Clone)]
pub struct HttpWorkOsClient {
    http: reqwest::Client,
    api_base: Url,
    client_id: String,
    api_key: String,
}

impl HttpWorkOsClient {
    /// Construct a production HTTP client. `client_id` and `api_key`
    /// come from the WorkOS dashboard.
    pub fn new(client_id: impl Into<String>, api_key: impl Into<String>) -> WorkOsResult<Self> {
        let client_id = client_id.into();
        let api_key = api_key.into();
        if client_id.is_empty() {
            return Err(WorkOsError::Config("client_id is empty".into()));
        }
        if api_key.is_empty() {
            return Err(WorkOsError::Config("api_key is empty".into()));
        }
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            http,
            api_base: Url::parse("https://api.workos.com").expect("static url"),
            client_id,
            api_key,
        })
    }

    /// Override the API base URL (for tests / staging).
    #[must_use]
    pub fn with_api_base(mut self, base: Url) -> Self {
        self.api_base = base;
        self
    }
}

#[async_trait]
impl WorkOsClient for HttpWorkOsClient {
    fn authorization_url(
        &self,
        redirect_uri: &str,
        state: Option<&str>,
        login_hint: Option<&str>,
    ) -> WorkOsResult<Url> {
        // AuthKit pattern: https://api.workos.com/user_management/authorize?...
        let mut url = self.api_base.join("/user_management/authorize")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("response_type", "code");
            qp.append_pair("client_id", &self.client_id);
            qp.append_pair("redirect_uri", redirect_uri);
            qp.append_pair("provider", "authkit");
            if let Some(s) = state {
                qp.append_pair("state", s);
            }
            if let Some(hint) = login_hint {
                qp.append_pair("login_hint", hint);
            }
        }
        Ok(url)
    }

    async fn authenticate_with_code(&self, _code: &str) -> WorkOsResult<AuthenticatedSession> {
        // Stub. The real implementation:
        //   1. POST /user_management/authenticate with {grant_type, code, client_id, client_secret}
        //   2. Parse the response into WorkOsUser + AccessToken + (optional) RefreshToken
        //   3. List memberships via GET /user_management/organization_memberships?user_id=
        //   4. Fetch each org via GET /organizations/{id}
        //   5. Assemble AuthenticatedSession.
        //
        // Stubbed because:
        //   - WorkOS is not provisioned yet (Bryan creates the team manually)
        //   - The exact endpoint shapes evolve; locking them in code before
        //     the team is provisioned would mean rewriting on first contact.
        //
        // Once WorkOS is provisioned, fill this in and uncomment the wiremock-based
        // test in tests/.
        Err(WorkOsError::Unexpected(
            "HttpWorkOsClient::authenticate_with_code not yet implemented — \
             pending WorkOS team provisioning".into(),
        ))
    }

    async fn refresh(&self, _refresh_token: &str) -> WorkOsResult<AuthenticatedSession> {
        Err(WorkOsError::Unexpected(
            "HttpWorkOsClient::refresh not yet implemented".into(),
        ))
    }

    async fn revoke_session(&self, _session_id: &str) -> WorkOsResult<()> {
        Err(WorkOsError::Unexpected(
            "HttpWorkOsClient::revoke_session not yet implemented".into(),
        ))
    }

    fn logout_url(&self, session_id: &str) -> WorkOsResult<Url> {
        let mut url = self.api_base.join("/user_management/sessions/logout")?;
        url.query_pairs_mut().append_pair("session_id", session_id);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_client_id() {
        let r = HttpWorkOsClient::new("", "key");
        assert!(matches!(r, Err(WorkOsError::Config(_))));
    }

    #[test]
    fn authorization_url_includes_required_params() {
        let c = HttpWorkOsClient::new("client_abc", "sk_test_xyz").unwrap();
        let u = c.authorization_url(
            "https://mukoko.com/auth/callback",
            Some("state-123"),
            Some("bryan@example.com"),
        ).unwrap();
        let q: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
        assert_eq!(q.get("client_id").map(String::as_str), Some("client_abc"));
        assert_eq!(q.get("response_type").map(String::as_str), Some("code"));
        assert_eq!(q.get("state").map(String::as_str), Some("state-123"));
        assert_eq!(q.get("login_hint").map(String::as_str), Some("bryan@example.com"));
    }
}
