//! The [`Session`] type — the in-memory session that gets serialised into
//! the platform JWT and consumed by every downstream service.
//!
//! Unlike [`crate::Person`], [`crate::Entity`], and [`crate::Membership`],
//! a session is never persisted. It exists only for the duration of an
//! authentication exchange or JWT validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::EntityType;

/// One entity claim within a session — what gets embedded in the
/// platform JWT's `entities` array.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityClaim {
    /// FK → identity.entity.id.
    pub id: Uuid,
    /// Family or organization.
    #[serde(rename = "type")]
    pub entity_type: EntityType,
    /// Display name of the entity.
    pub name: String,
    /// The role this person holds within this entity.
    pub role: String,
    /// Permissions resolved from role + membership.permissions.
    pub permissions: Vec<String>,
}

/// The active context a request is being made under.
///
/// A person has many memberships but acts within exactly one at a time.
/// The active context is set at signin (defaults to the family-of-one
/// or the most recently active entity) and switched via `/entity/switch`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveContext {
    /// Active entity id.
    pub entity_id: Uuid,
    /// Active entity type.
    pub entity_type: EntityType,
    /// Active role within that entity.
    pub role: String,
}

/// A complete authenticated session — the in-memory shape that gets
/// signed into a platform JWT.
///
/// Every downstream service receives this shape (via JWT decode) and
/// authorises requests based on `active_context.role` plus the
/// permissions of the matching `EntityClaim` in `entities`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    /// FK → identity.person.id.
    pub person_id: Uuid,

    /// Cached display name (avoids a downstream Supabase lookup for
    /// rendering "Hi, $name" in UIs).
    pub name: String,

    /// Cached email (downstream services may need it for auditing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Email verification flag at session-mint time.
    pub email_verified: bool,

    /// WorkOS user id, for any service that needs to call WorkOS
    /// directly (rare — should generally go through the gateway).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workos_user_id: Option<String>,

    /// All visible memberships at session-mint time.
    pub entities: Vec<EntityClaim>,

    /// The currently active entity context.
    pub active_context: ActiveContext,

    /// When the session was minted.
    pub issued_at: DateTime<Utc>,

    /// When the session expires.
    pub expires_at: DateTime<Utc>,
}

impl Session {
    /// Returns the [`EntityClaim`] for the currently active entity, if
    /// it exists in `entities`. A well-formed session always satisfies
    /// this — but defensive code paths may want to verify.
    #[must_use]
    pub fn active_entity(&self) -> Option<&EntityClaim> {
        self.entities.iter().find(|e| e.id == self.active_context.entity_id)
    }

    /// Switches the active context to the given entity id, if the
    /// session has a matching membership. Returns true on success.
    pub fn switch_entity(&mut self, entity_id: Uuid) -> bool {
        if let Some(claim) = self.entities.iter().find(|e| e.id == entity_id) {
            self.active_context = ActiveContext {
                entity_id: claim.id,
                entity_type: claim.entity_type,
                role: claim.role.clone(),
            };
            true
        } else {
            false
        }
    }

    /// Returns true if `permission` is granted in the active entity context.
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        let Some(active) = self.active_entity() else { return false };
        active.permissions.iter().any(|p| p == permission || p == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family_session() -> Session {
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
    fn active_entity_resolves() {
        let s = family_session();
        assert!(s.active_entity().is_some());
    }

    #[test]
    fn switch_entity_to_unknown_returns_false() {
        let mut s = family_session();
        let unknown = Uuid::now_v7();
        assert!(!s.switch_entity(unknown));
    }

    #[test]
    fn wildcard_permission_matches_any() {
        let mut s = family_session();
        s.entities[0].permissions = vec!["*".into()];
        s.active_context.role = "owner".into();
        assert!(s.has_permission("anything.at.all"));
    }
}
