//! The [`Membership`] domain type — the universal join between
//! [`crate::Person`] and [`crate::Entity`].
//!
//! Maps to `entity.membership`. Per v4.0.3 every signup creates one
//! membership of role [`MembershipRole::Head`] in the new family-of-one
//! entity. Subsequent memberships are created through invitation flows
//! (family additions, organization onboarding) handled at the auth
//! gateway level.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;

/// Locked membership status values per v4.0.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MembershipStatus {
    /// Invited, no acceptance yet.
    Pending,
    /// Invitation sent, awaiting accept.
    Invited,
    /// Active member.
    Active,
    /// Temporarily suspended.
    Suspended,
    /// Departed voluntarily.
    Departed,
    /// Removed by admin.
    Removed,
}

impl fmt::Display for MembershipStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Invited => "invited",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Departed => "departed",
            Self::Removed => "removed",
        };
        f.write_str(s)
    }
}

impl FromStr for MembershipStatus {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "invited" => Ok(Self::Invited),
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "departed" => Ok(Self::Departed),
            "removed" => Ok(Self::Removed),
            other => Err(CoreError::InvalidMembershipStatus(other.to_string())),
        }
    }
}

/// Common membership roles.
///
/// Stored as free text in the database (`entity.membership.role`) for
/// flexibility — different cultural contexts use different role names.
/// This enum captures the roles the platform code paths recognise
/// directly; any other string is treated as a custom role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MembershipRole {
    /// Head of household (family entities) — also default for the
    /// family-of-one bootstrap at signup.
    Head,
    /// Adult partner in a family entity.
    Partner,
    /// Child or dependant in a family entity.
    Dependant,
    /// Elder in a family entity (cultural context).
    Elder,
    /// Owner of an organisation entity.
    Owner,
    /// Administrator of an organisation entity.
    Admin,
    /// Generic member.
    Member,
    /// Staff role in an organisation.
    Staff,
}

impl fmt::Display for MembershipRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Head => "head",
            Self::Partner => "partner",
            Self::Dependant => "dependant",
            Self::Elder => "elder",
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Staff => "staff",
        };
        f.write_str(s)
    }
}

/// A person's membership in an entity.
///
/// One row per (person, entity) pair — uniqueness enforced at the
/// database layer. Permissions are stored as JSON for flexibility;
/// the gateway resolves them to a concrete capability set when
/// minting the platform JWT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Membership {
    /// UUIDv7.
    pub id: Uuid,

    /// FK → identity.person.id.
    pub person_id: Uuid,

    /// FK → identity.entity.id.
    pub entity_id: Uuid,

    /// Role within the entity. Free text; common values via
    /// [`MembershipRole`].
    pub role: String,

    /// Optional human-readable role title (e.g. "Head of household",
    /// "Chief Marketing Officer").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_title: Option<String>,

    /// RBAC permissions as JSON. Merged with role defaults at JWT mint.
    pub permissions: serde_json::Value,

    /// Lifecycle status.
    pub status: MembershipStatus,

    /// True for the person's "default" entity context.
    pub is_primary: bool,

    /// WorkOS OrganizationMembership id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workos_membership_id: Option<String>,

    /// Person who issued the invitation, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_by: Option<Uuid>,

    /// Funnel provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_project: Option<String>,

    /// PRIMARY ack flag.
    pub synced_to_primary: bool,

    /// PRIMARY ack timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_synced_at: Option<DateTime<Utc>>,

    /// When the membership became active.
    pub joined_at: DateTime<Utc>,

    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Row last-modified timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Membership {
    /// Creates a head-of-family membership for the given person and
    /// family entity. Used by the signup family-of-one bootstrap.
    #[must_use]
    pub fn new_family_head(
        person_id: Uuid,
        entity_id: Uuid,
        source_project: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            person_id,
            entity_id,
            role: MembershipRole::Head.to_string(),
            role_title: Some("Head of household".to_string()),
            permissions: serde_json::json!({}),
            status: MembershipStatus::Active,
            is_primary: true,
            workos_membership_id: None,
            invited_by: None,
            source_project,
            synced_to_primary: false,
            primary_synced_at: None,
            joined_at: now,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if the membership is in a state that should be
    /// included in the platform JWT's `entities` claim.
    #[must_use]
    pub fn is_jwt_visible(&self) -> bool {
        matches!(self.status, MembershipStatus::Active | MembershipStatus::Suspended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        for s in [
            MembershipStatus::Pending,
            MembershipStatus::Active,
            MembershipStatus::Removed,
        ] {
            let parsed: MembershipStatus = s.to_string().parse().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn family_head_invariants() {
        let p = Uuid::now_v7();
        let e = Uuid::now_v7();
        let m = Membership::new_family_head(p, e, Some("mukoko_news_db".into()));
        assert_eq!(m.person_id, p);
        assert_eq!(m.entity_id, e);
        assert_eq!(m.role, "head");
        assert_eq!(m.status, MembershipStatus::Active);
        assert!(m.is_primary);
        assert!(m.is_jwt_visible());
    }
}
