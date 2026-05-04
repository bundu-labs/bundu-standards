//! Error types for the core domain layer.

use thiserror::Error;

/// Errors that can arise from validating or constructing core domain types.
///
/// Operational errors (network, database, WorkOS) live in their respective
/// crates — this enum only models the *domain* invariants.
#[derive(Debug, Error)]
pub enum CoreError {
    /// An entity_type value was not one of the locked enum values.
    #[error("invalid entity type: {0}")]
    InvalidEntityType(String),

    /// A membership status value was not one of the locked enum values.
    #[error("invalid membership status: {0}")]
    InvalidMembershipStatus(String),

    /// A required field was missing or empty.
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    /// A field value violated a domain rule.
    #[error("invalid field {field}: {reason}")]
    InvalidField {
        /// The name of the offending field.
        field: &'static str,
        /// A human-readable description of the violation.
        reason: String,
    },
}

/// Convenient `Result` alias for the core layer.
pub type CoreResult<T> = std::result::Result<T, CoreError>;
