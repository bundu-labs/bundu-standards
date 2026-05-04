//! Bundu Identity core domain types.
//!
//! These types model the identity layer of the Bundu Family ecosystem as
//! locked in Architecture v4.0.3. They are intentionally framework-free —
//! no axum, no sqlx, no reqwest. Pure data structures plus their invariants.
//!
//! The mapping to the database (Architecture v4.0.3):
//! - [`Person`]      ↔ `identity.person`
//! - [`Entity`]      ↔ `identity.entity`
//! - [`Membership`]  ↔ `entity.membership`
//! - [`Session`]     ↔ in-memory only (platform JWT claims)
//!
//! The mapping to WorkOS:
//! - [`Person.workos_user_id`]      ↔ WorkOS User
//! - [`Entity.workos_org_id`]       ↔ WorkOS Organization
//! - [`Membership.workos_membership_id`] ↔ WorkOS OrganizationMembership

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod entity;
pub mod membership;
pub mod person;
pub mod session;

pub use error::{CoreError, CoreResult};
pub use entity::{Entity, EntityType};
pub use membership::{Membership, MembershipRole, MembershipStatus};
pub use person::Person;
pub use session::{ActiveContext, EntityClaim, Session};
