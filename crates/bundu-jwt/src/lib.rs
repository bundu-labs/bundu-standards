//! Platform JWT minting and validation.
//!
//! This crate defines the canonical platform JWT contract — the token
//! that every service across the Bundu ecosystem trusts. The format is
//! versioned via the `iss` claim and structured to allow algorithm
//! migration (HS256 today, ML-DSA in future per v4.0.3 post-quantum
//! roadmap) without breaking downstream consumers.
//!
//! Downstream services validate using any standard JWT library in their
//! language — they do not depend on this crate. This crate is consumed
//! by the auth gateway (for minting) and any Rust service that wants
//! strongly-typed validation (e.g. NTL, SiafuDB).

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;

use bundu_identity_core::{ActiveContext, EntityClaim, Session};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// The issuer string locked for all platform JWTs.
pub const PLATFORM_JWT_ISSUER: &str = "https://id.mukoko.com";

/// Default audiences. Downstream services SHOULD validate that they
/// appear in this list (e.g. `mukoko` for consumer apps, `nyuchi` for
/// enterprise products).
pub const DEFAULT_AUDIENCES: &[&str] = &["mukoko", "nyuchi"];

/// Default token lifetime for web sessions.
pub const WEB_SESSION_LIFETIME_SECONDS: i64 = 30 * 24 * 60 * 60;

/// Default token lifetime for native (mobile, desktop) sessions.
pub const NATIVE_SESSION_LIFETIME_SECONDS: i64 = 365 * 24 * 60 * 60;

/// Errors raised by the JWT layer.
#[derive(Debug, Error)]
pub enum JwtError {
    /// Underlying jsonwebtoken error.
    #[error("jwt encode/decode error: {0}")]
    Library(#[from] jsonwebtoken::errors::Error),
    /// Active entity not found in the entities list — malformed session.
    #[error("active entity {0} not found in session entities")]
    ActiveEntityMissing(Uuid),
    /// JWT signing key is missing or empty.
    #[error("jwt signing key not configured")]
    MissingSigningKey,
}

/// Result alias for the JWT crate.
pub type JwtResult<T> = std::result::Result<T, JwtError>;

/// The wire format of a platform JWT — what gets encoded into the
/// signed token body. Matches the contract documented in the README
/// of this repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformClaims {
    /// Issuer — locked to [`PLATFORM_JWT_ISSUER`].
    pub iss: String,
    /// Subject — `identity.person.id`.
    pub sub: Uuid,
    /// Audience.
    pub aud: Vec<String>,
    /// Issued-at (Unix seconds).
    pub iat: i64,
    /// Expiry (Unix seconds).
    pub exp: i64,
    /// JWT id — UUIDv7 for time-ordered uniqueness.
    pub jti: Uuid,

    /// Cached display name.
    pub name: String,
    /// Cached email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Email verification flag at mint time.
    pub email_verified: bool,
    /// WorkOS user id (cross-system reference).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workos_user_id: Option<String>,

    /// All visible entity memberships at mint time.
    pub entities: Vec<EntityClaim>,

    /// Active entity id.
    pub active_entity_id: Uuid,
    /// Active entity type.
    pub active_entity_type: String,
    /// Active role.
    pub active_role: String,
}

impl PlatformClaims {
    /// Convert claims back into a [`Session`] for in-process use.
    /// Reverse of [`Self::from_session`].
    #[must_use]
    pub fn into_session(self) -> Session {
        let active_context = ActiveContext {
            entity_id: self.active_entity_id,
            entity_type: self.active_entity_type.parse().unwrap_or(
                bundu_identity_core::EntityType::Family
            ),
            role: self.active_role,
        };
        Session {
            person_id: self.sub,
            name: self.name,
            email: self.email,
            email_verified: self.email_verified,
            workos_user_id: self.workos_user_id,
            entities: self.entities,
            active_context,
            issued_at: DateTime::<Utc>::from_timestamp(self.iat, 0).unwrap_or_else(Utc::now),
            expires_at: DateTime::<Utc>::from_timestamp(self.exp, 0).unwrap_or_else(Utc::now),
        }
    }

    /// Build claims from a session. Validates session consistency.
    fn from_session(session: &Session) -> JwtResult<Self> {
        if session.active_entity().is_none() {
            return Err(JwtError::ActiveEntityMissing(session.active_context.entity_id));
        }
        Ok(Self {
            iss: PLATFORM_JWT_ISSUER.to_string(),
            sub: session.person_id,
            aud: DEFAULT_AUDIENCES.iter().map(|s| (*s).to_string()).collect(),
            iat: session.issued_at.timestamp(),
            exp: session.expires_at.timestamp(),
            jti: Uuid::now_v7(),
            name: session.name.clone(),
            email: session.email.clone(),
            email_verified: session.email_verified,
            workos_user_id: session.workos_user_id.clone(),
            entities: session.entities.clone(),
            active_entity_id: session.active_context.entity_id,
            active_entity_type: session.active_context.entity_type.to_string(),
            active_role: session.active_context.role.clone(),
        })
    }
}

/// JWT signer and verifier using a shared HS256 secret.
///
/// Phase 1 of the post-quantum roadmap (per v4.0.3 amendment) keeps
/// HS256 — symmetric, not vulnerable to Shor's algorithm. Phase 2+
/// migrates to ML-DSA when the toolchain matures. The signer abstraction
/// here is intentionally minimal so the algorithm can be swapped without
/// touching the call sites.
#[derive(Clone)]
pub struct PlatformJwtSigner {
    encoding_key: Arc<EncodingKey>,
    decoding_key: Arc<DecodingKey>,
    algorithm: Algorithm,
    issuer: String,
    audiences: Vec<String>,
}

impl PlatformJwtSigner {
    /// Construct a signer/verifier from a shared HMAC secret.
    ///
    /// The secret must be at least 32 bytes; shorter secrets are
    /// rejected to avoid weak keys.
    pub fn new_hs256(secret: impl AsRef<[u8]>) -> JwtResult<Self> {
        let bytes = secret.as_ref();
        if bytes.len() < 32 {
            return Err(JwtError::MissingSigningKey);
        }
        Ok(Self {
            encoding_key: Arc::new(EncodingKey::from_secret(bytes)),
            decoding_key: Arc::new(DecodingKey::from_secret(bytes)),
            algorithm: Algorithm::HS256,
            issuer: PLATFORM_JWT_ISSUER.to_string(),
            audiences: DEFAULT_AUDIENCES.iter().map(|s| (*s).to_string()).collect(),
        })
    }

    /// Mint a JWT for the given session.
    pub fn mint(&self, session: &Session) -> JwtResult<String> {
        let claims = PlatformClaims::from_session(session)?;
        let header = Header::new(self.algorithm);
        encode(&header, &claims, &self.encoding_key).map_err(JwtError::from)
    }

    /// Verify a JWT and return its claims.
    ///
    /// Validates issuer, audience, expiry, and signature. The returned
    /// claims are guaranteed to satisfy the contract.
    pub fn verify(&self, token: &str) -> JwtResult<PlatformClaims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&self.audiences);
        validation.validate_exp = true;
        let token_data = decode::<PlatformClaims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bundu_identity_core::EntityType;

    fn session_fixture() -> Session {
        let entity_id = Uuid::now_v7();
        let claim = EntityClaim {
            id: entity_id,
            entity_type: EntityType::Family,
            name: "Bryan's family".into(),
            role: "head".into(),
            permissions: vec!["family.manage".into()],
        };
        Session {
            person_id: Uuid::now_v7(),
            name: "Bryan".into(),
            email: Some("bryan@example.com".into()),
            email_verified: true,
            workos_user_id: Some("user_abc".into()),
            entities: vec![claim],
            active_context: ActiveContext {
                entity_id,
                entity_type: EntityType::Family,
                role: "head".into(),
            },
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(30),
        }
    }

    #[test]
    fn rejects_short_secret() {
        let r = PlatformJwtSigner::new_hs256("too-short");
        assert!(r.is_err());
    }

    #[test]
    fn mint_and_verify_roundtrip() {
        let signer = PlatformJwtSigner::new_hs256(
            "an-acceptable-32-byte-secret-for-testing!!"
        ).unwrap();
        let session = session_fixture();
        let token = signer.mint(&session).unwrap();
        let claims = signer.verify(&token).unwrap();
        assert_eq!(claims.sub, session.person_id);
        assert_eq!(claims.active_entity_id, session.active_context.entity_id);
        assert_eq!(claims.iss, PLATFORM_JWT_ISSUER);
    }

    #[test]
    fn tampered_token_rejected() {
        let signer = PlatformJwtSigner::new_hs256(
            "an-acceptable-32-byte-secret-for-testing!!"
        ).unwrap();
        let session = session_fixture();
        let mut token = signer.mint(&session).unwrap();
        // Flip a character in the signature
        let last = token.pop().unwrap();
        token.push(if last == 'a' { 'b' } else { 'a' });
        assert!(signer.verify(&token).is_err());
    }
}
