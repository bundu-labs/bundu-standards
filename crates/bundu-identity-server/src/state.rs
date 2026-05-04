//! Shared application state — the dependency-injected handles every
//! route handler reads from.

use std::sync::Arc;

use bundu_jwt::PlatformJwtSigner;
use bundu_service_bus::Publisher;
use bundu_supabase::IdentityResolver;
use bundu_workos::WorkOsClient;

use crate::config::AppConfig;

/// Shared, immutable state for the axum app.
///
/// Trait objects rather than concrete types so the gateway is testable
/// against mocks without rewriting handlers.
pub struct AppState {
    /// The full config (occasionally useful in handlers — environment
    /// label, instance id, redirect_uri).
    pub config: AppConfig,
    /// WorkOS client.
    pub workos: Arc<dyn WorkOsClient>,
    /// Supabase identity resolver.
    pub identity: Arc<dyn IdentityResolver>,
    /// JWT signer/verifier.
    pub jwt: PlatformJwtSigner,
    /// Service bus publisher.
    pub bus: Arc<dyn Publisher>,
}
