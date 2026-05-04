//! The [`Entity`] domain type — umbrella for families and organizations.
//!
//! Maps to `identity.entity` in PRIMARY and all sub-projects. Per v4.0.3,
//! every signup creates a family-of-one entity automatically, and
//! organizations are also entities (mirrored from `business.organization`
//! during the v4.0.3 transition).
//!
//! Schema.org: serialises as `Organization`, with `additionalType` set to
//! `bundu:Family` for family entities (the `bundu:` namespace is the
//! canonical place for cultural Schema.org extensions per v4.0.3).

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;

/// The two locked entity types per v4.0.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// Default at signup. Schema.org: Organization with
    /// additionalType `bundu:Family`.
    Family,

    /// Verification-gated. Schema.org: Organization (and subtypes —
    /// LocalBusiness, NewsMediaOrganization, etc.).
    Organization,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Family => write!(f, "family"),
            Self::Organization => write!(f, "organization"),
        }
    }
}

impl FromStr for EntityType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "family" => Ok(Self::Family),
            "organization" => Ok(Self::Organization),
            other => Err(CoreError::InvalidEntityType(other.to_string())),
        }
    }
}

impl EntityType {
    /// Returns the canonical Schema.org `additionalType` value for this
    /// entity type. Family entities use `bundu:Family`; organisation
    /// entities default to `bundu:Business` and may be refined upstream.
    #[must_use]
    pub fn default_additional_type(self) -> &'static str {
        match self {
            Self::Family => "bundu:Family",
            Self::Organization => "bundu:Business",
        }
    }
}

/// An entity — either a family or a formal organisation.
///
/// Mirrors `identity.entity` row-for-row. The `metadata` field carries
/// arbitrary additional fields that don't merit their own column
/// (e.g. capability flags carried over from `business.organization`
/// during the v4.0.3 mirror migration).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    /// UUIDv7 (RFC 9562).
    pub id: Uuid,

    /// Discriminator — family or organization.
    pub entity_type: EntityType,

    /// Schema.org `name`.
    pub name: String,

    /// Schema.org `alternateName`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_name: Option<String>,

    /// Schema.org `description`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Schema.org `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Schema.org `logo`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,

    /// URL-safe slug. Unique across all entities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// WorkOS Organization id. Cross-system reference.
    /// Set by the auth gateway when the entity is provisioned in WorkOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workos_org_id: Option<String>,

    /// Schema.org `additionalType` — e.g. `bundu:Family`,
    /// `bundu:Business`, `bundu:Clinic`. Defaults via
    /// [`EntityType::default_additional_type`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_type: Option<String>,

    /// Verification tier (0..=4) per the system.verification_tier ladder.
    pub verification_tier_level: i16,

    /// When the entity was last verified at its current tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<DateTime<Utc>>,

    /// Denormalised count of active members.
    pub member_count: i32,

    /// ScyllaDB document id (where content lives, per Layer 3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scylladb_doc_id: Option<String>,

    /// CouchDB sync-pipe document id (Layer 4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub couchdb_doc_id: Option<String>,

    /// Free-form metadata. JSON object with arbitrary keys.
    /// Conventionally includes `capabilities` (for organizations
    /// migrated from business.organization), `auto_created` (for
    /// family-of-one entities), and any sync provenance.
    pub metadata: serde_json::Value,

    /// Funnel provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_project: Option<String>,

    /// PRIMARY ack flag.
    pub synced_to_primary: bool,

    /// PRIMARY ack timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_synced_at: Option<DateTime<Utc>>,

    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Row last-modified timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    /// Creates a fresh family-of-one entity for the given person.
    /// Used by the signup flow.
    #[must_use]
    pub fn new_family(name: impl Into<String>, source_project: Option<String>) -> Self {
        let now = Utc::now();
        let metadata = serde_json::json!({
            "auto_created": true,
            "reason": "signup family-of-one bootstrap (v4.0.3)"
        });
        Self {
            id: Uuid::now_v7(),
            entity_type: EntityType::Family,
            name: name.into(),
            alternate_name: None,
            description: None,
            url: None,
            logo: None,
            slug: None,
            workos_org_id: None,
            additional_type: Some("bundu:Family".to_string()),
            verification_tier_level: 0,
            verified_at: None,
            member_count: 1,
            scylladb_doc_id: None,
            couchdb_doc_id: None,
            metadata,
            source_project,
            synced_to_primary: false,
            primary_synced_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if this entity may directly hold a wallet, accept
    /// payments, or otherwise transact in its own name. Family entities
    /// transact via their `head` member; organization entities transact
    /// directly once verified.
    #[must_use]
    pub fn can_transact_directly(&self) -> bool {
        matches!(self.entity_type, EntityType::Organization) && self.verification_tier_level >= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_roundtrip() {
        assert_eq!(EntityType::Family.to_string(), "family");
        assert_eq!(EntityType::Organization.to_string(), "organization");
        assert_eq!("family".parse::<EntityType>().unwrap(), EntityType::Family);
        assert_eq!("organization".parse::<EntityType>().unwrap(), EntityType::Organization);
        assert!("partnership".parse::<EntityType>().is_err());
    }

    #[test]
    fn new_family_sets_invariants() {
        let f = Entity::new_family("Bryan's family", Some("mukoko_news_db".into()));
        assert_eq!(f.entity_type, EntityType::Family);
        assert_eq!(f.additional_type.as_deref(), Some("bundu:Family"));
        assert_eq!(f.verification_tier_level, 0);
        assert_eq!(f.member_count, 1);
        assert!(!f.synced_to_primary);
        assert!(!f.can_transact_directly());
    }

    #[test]
    fn family_cannot_transact_directly() {
        let f = Entity::new_family("Test", None);
        assert!(!f.can_transact_directly());
    }
}
