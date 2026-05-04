//! The [`Person`] domain type — one human authenticating on the platform.
//!
//! Maps to `identity.person` in PRIMARY (and slim mirrors in sub-projects).
//! This is the *human* in the v4.0.3 identity model. A person belongs to
//! one or more entities via [`crate::Membership`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single human authenticating on the Bundu platform.
///
/// The Schema.org type is `Person`. Every person has at minimum a UUID
/// identifier, a display name, and an authentication anchor (email,
/// telephone, or WorkOS user id). All other fields are optional — the
/// platform progressively enriches a person's profile after signup.
///
/// Per v4.0.3, every person is automatically a member of a family-of-one
/// entity at signup. This invariant is enforced at the database layer
/// (via the auth gateway's signup logic), not in this struct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    /// UUIDv7 (RFC 9562) — time-ordered for B-tree health.
    pub id: Uuid,

    /// WorkOS User id. The authoritative cross-system reference.
    /// Nullable until the person has authenticated through WorkOS at
    /// least once (e.g. legacy Stytch users awaiting migration).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workos_user_id: Option<String>,

    /// Legacy Stytch user id. Preserved during the WorkOS migration window
    /// per v4.0.3, dropped in a future migration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stytch_user_id: Option<String>,

    /// Schema.org `name` — the person's full display name.
    pub name: String,

    /// Schema.org `givenName` — first name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// Schema.org `familyName` — last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    /// Schema.org `email`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Schema.org `telephone` (E.164 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telephone: Option<String>,

    /// Schema.org `image` — profile picture URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Email verification flag.
    pub email_verified: bool,

    /// Phone verification flag.
    pub phone_verified: bool,

    /// Funnel provenance — which sub-project created this row.
    /// `None` indicates PRIMARY origin or pre-funnel data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_project: Option<String>,

    /// True once PRIMARY has acknowledged this row via the service bus.
    pub synced_to_primary: bool,

    /// When PRIMARY acknowledged this row (None until first sync).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_synced_at: Option<DateTime<Utc>>,

    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Row last-modified timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Person {
    /// Returns true if the person has at least one authentication anchor.
    /// Used as a precondition before minting a platform JWT.
    #[must_use]
    pub fn has_auth_anchor(&self) -> bool {
        self.workos_user_id.is_some()
            || self.stytch_user_id.is_some()
            || self.email.is_some()
            || self.telephone.is_some()
    }

    /// Returns the best display name available, falling back through
    /// `name` → `given_name` → email local-part → "Unknown".
    #[must_use]
    pub fn display_name(&self) -> String {
        if !self.name.is_empty() {
            return self.name.clone();
        }
        if let Some(given) = &self.given_name {
            return given.clone();
        }
        if let Some(email) = &self.email {
            if let Some((local, _)) = email.split_once('@') {
                return local.to_string();
            }
        }
        "Unknown".to_string()
    }

    /// Suggested family-of-one name for this person, used at signup
    /// when auto-creating their default entity.
    /// Format: `{given_name | first word of name}'s family`
    #[must_use]
    pub fn suggested_family_name(&self) -> String {
        let head = if let Some(given) = &self.given_name {
            given.clone()
        } else if !self.name.is_empty() {
            self.name.split_whitespace().next().unwrap_or("Unknown").to_string()
        } else {
            self.display_name()
        };
        format!("{head}'s family")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str, given: Option<&str>, email: Option<&str>) -> Person {
        Person {
            id: Uuid::now_v7(),
            workos_user_id: None,
            stytch_user_id: None,
            name: name.to_string(),
            given_name: given.map(String::from),
            family_name: None,
            email: email.map(String::from),
            telephone: None,
            image: None,
            email_verified: false,
            phone_verified: false,
            source_project: None,
            synced_to_primary: false,
            primary_synced_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn suggested_family_name_uses_given_name_when_present() {
        let p = fixture("Bryan Fawcett", Some("Bryan"), None);
        assert_eq!(p.suggested_family_name(), "Bryan's family");
    }

    #[test]
    fn suggested_family_name_falls_back_to_first_word_of_name() {
        let p = fixture("Bryan Fawcett", None, None);
        assert_eq!(p.suggested_family_name(), "Bryan's family");
    }

    #[test]
    fn suggested_family_name_falls_back_to_email_local_part() {
        let p = fixture("", None, Some("bryan@example.com"));
        assert_eq!(p.suggested_family_name(), "bryan's family");
    }

    #[test]
    fn has_auth_anchor_requires_some_anchor() {
        let mut p = fixture("Test", None, None);
        assert!(!p.has_auth_anchor());
        p.email = Some("a@b.com".into());
        assert!(p.has_auth_anchor());
    }
}
