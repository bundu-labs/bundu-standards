//! Supabase REST client for the identity resolver.
//!
//! Authenticates as `service_role` per memory #26 (API-only access pattern,
//! anon/authenticated grants stripped, service_role is operative). Upserts
//! `identity.person`, `identity.entity`, and `entity.membership` on signup.
//!
//! Calls PostgREST endpoints exposed by Supabase under `/rest/v1/`. Could
//! also call edge functions if the gateway needs to delegate complex
//! resolution logic — for now, direct PostgREST is sufficient.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use async_trait::async_trait;
use bundu_identity_core::{Entity, Membership, Person};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

/// Errors from the Supabase layer.
#[derive(Debug, Error)]
pub enum SupabaseError {
    /// Configuration is missing or invalid.
    #[error("supabase config error: {0}")]
    Config(String),
    /// HTTP error talking to Supabase.
    #[error("supabase http error: {0}")]
    Http(#[from] reqwest::Error),
    /// URL parse error.
    #[error("supabase url parse error: {0}")]
    Url(#[from] url::ParseError),
    /// JSON parse error.
    #[error("supabase json error: {0}")]
    Json(#[from] serde_json::Error),
    /// The expected row was not found (e.g. resolving by workos_user_id).
    #[error("supabase row not found")]
    NotFound,
    /// An upsert returned an unexpected response shape.
    #[error("supabase unexpected response: {0}")]
    Unexpected(String),
}

/// Result alias.
pub type SupabaseResult<T> = std::result::Result<T, SupabaseError>;

/// The bundle returned by [`IdentityResolver::resolve_or_create`] after
/// a successful WorkOS authentication.
#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    /// The platform-side person row.
    pub person: Person,
    /// All entities the person belongs to (full rows).
    pub entities: Vec<Entity>,
    /// All memberships the person holds.
    pub memberships: Vec<Membership>,
    /// Default active entity — usually the family-of-one, may be a
    /// recently-active organisation if the gateway tracks that.
    pub active_entity_id: Uuid,
}

/// The contract the gateway server uses to talk to the identity store.
///
/// Implementations:
/// - [`HttpIdentityResolver`] — production, talks to Supabase PostgREST.
/// - In-memory mock (in tests).
#[async_trait]
pub trait IdentityResolver: Send + Sync {
    /// Resolve a WorkOS user to a platform [`ResolvedIdentity`], creating
    /// the person row, family-of-one entity, and head membership if they
    /// don't already exist (signup path).
    async fn resolve_or_create(
        &self,
        workos_user_id: &str,
        email: &str,
        name: &str,
        given_name: Option<&str>,
        family_name: Option<&str>,
    ) -> SupabaseResult<ResolvedIdentity>;

    /// Look up an existing person by WorkOS user id without creating.
    /// Returns [`SupabaseError::NotFound`] if no row exists.
    async fn find_by_workos_user_id(&self, workos_user_id: &str) -> SupabaseResult<Person>;

    /// List all entities the given person is a member of (regardless
    /// of membership status — filtering is done at JWT-mint time).
    async fn list_entities_for_person(&self, person_id: Uuid) -> SupabaseResult<Vec<Entity>>;

    /// List all memberships for the given person.
    async fn list_memberships_for_person(
        &self,
        person_id: Uuid,
    ) -> SupabaseResult<Vec<Membership>>;
}

/// HTTP implementation of [`IdentityResolver`] talking to Supabase PostgREST.
#[derive(Clone)]
pub struct HttpIdentityResolver {
    http: reqwest::Client,
    base: Url,
    service_role_key: String,
}

impl HttpIdentityResolver {
    /// Construct a resolver targeting a specific Supabase project.
    /// `base_url` is the project's base (e.g.
    /// `https://tdcpuzqyoodrdsxldgsh.supabase.co`).
    pub fn new(base_url: impl AsRef<str>, service_role_key: impl Into<String>) -> SupabaseResult<Self> {
        let base = Url::parse(base_url.as_ref())?;
        let key = service_role_key.into();
        if key.is_empty() {
            return Err(SupabaseError::Config("service role key is empty".into()));
        }
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;
        Ok(Self { http, base, service_role_key: key })
    }
}

#[async_trait]
impl IdentityResolver for HttpIdentityResolver {
    async fn resolve_or_create(
        &self,
        _workos_user_id: &str,
        _email: &str,
        _name: &str,
        _given_name: Option<&str>,
        _family_name: Option<&str>,
    ) -> SupabaseResult<ResolvedIdentity> {
        // Stub. The real implementation:
        //   1. POST /rest/v1/rpc/resolve_or_create_identity (a Postgres function we'll add)
        //      with the WorkOS user_id and profile fields. The function:
        //        - SELECTs identity.person by workos_user_id (the unique index added in v4.0.3)
        //        - If found: returns the existing row + entities + memberships
        //        - If not: INSERTs identity.person, creates family-of-one entity,
        //          creates head membership, all in one transaction, then returns the bundle
        //   2. Parse the JSON response into ResolvedIdentity.
        //
        // Doing this in a single Postgres function is preferable to multiple
        // round-trips because (a) the family-of-one bootstrap must be atomic
        // and (b) cross-table reads on the same connection are cheap.
        //
        // The function lives in /migrations/ and is added in a separate migration
        // once the gateway is wired and tested end-to-end.
        Err(SupabaseError::Unexpected(
            "HttpIdentityResolver::resolve_or_create not yet implemented — \
             pending resolve_or_create_identity SQL function".into(),
        ))
    }

    async fn find_by_workos_user_id(&self, _workos_user_id: &str) -> SupabaseResult<Person> {
        // Stub. The real implementation:
        //   GET /rest/v1/identity.person?workos_user_id=eq.{id}&select=*
        //   with apikey + Authorization: Bearer {service_role_key}
        Err(SupabaseError::NotFound)
    }

    async fn list_entities_for_person(&self, _person_id: Uuid) -> SupabaseResult<Vec<Entity>> {
        // Stub. The real implementation:
        //   GET /rest/v1/identity.entity?id=in.(<membership entity_ids>)
        Ok(Vec::new())
    }

    async fn list_memberships_for_person(
        &self,
        _person_id: Uuid,
    ) -> SupabaseResult<Vec<Membership>> {
        // Stub. The real implementation:
        //   GET /rest/v1/entity.membership?person_id=eq.{id}&status=in.(active,suspended)
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_service_role_key() {
        let r = HttpIdentityResolver::new("https://example.supabase.co", "");
        assert!(matches!(r, Err(SupabaseError::Config(_))));
    }

    #[test]
    fn accepts_valid_config() {
        let r = HttpIdentityResolver::new("https://example.supabase.co", "key");
        assert!(r.is_ok());
    }
}
