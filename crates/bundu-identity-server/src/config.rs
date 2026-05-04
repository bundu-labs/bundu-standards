//! Configuration loaded from environment variables.

use serde::Deserialize;

/// Top-level application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// `0.0.0.0:8080` style listen address.
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    /// `info`, `debug`, `trace`.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// `production`, `staging`, `development`.
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Instance identifier (e.g. `auth-gw-johannesburg-1`). Used in
    /// service bus envelopes.
    #[serde(default = "default_instance")]
    pub instance: String,

    /// WorkOS settings.
    pub workos: WorkOsConfig,

    /// Supabase settings.
    pub supabase: SupabaseConfig,

    /// JWT signing settings.
    pub jwt: JwtConfig,

    /// Cloudflare Queues settings.
    pub cloudflare: CloudflareConfig,
}

/// WorkOS-specific config.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkOsConfig {
    /// WorkOS client id (from dashboard).
    pub client_id: String,
    /// WorkOS API secret key.
    pub api_key: String,
    /// Default redirect URI used for AuthKit.
    pub redirect_uri: String,
}

/// Supabase-specific config.
#[derive(Debug, Clone, Deserialize)]
pub struct SupabaseConfig {
    /// Project URL — e.g. `https://tdcpuzqyoodrdsxldgsh.supabase.co`.
    pub url: String,
    /// Service role key (NEVER exposed to clients).
    pub service_role_key: String,
}

/// JWT signing config.
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// HMAC secret for HS256. ML-DSA migration tracked separately
    /// per v4.0.3 post-quantum roadmap.
    pub signing_secret: String,
}

/// Cloudflare Queues config.
#[derive(Debug, Clone, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare account id.
    pub account_id: String,
    /// API token with Queues:Edit scope.
    pub queues_api_token: String,
}

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_environment() -> String {
    "development".to_string()
}
fn default_instance() -> String {
    format!("local-{}", std::process::id())
}

impl AppConfig {
    /// Load config from `.env` (if present) plus environment variables.
    /// Env var names are uppercased and prefixed: `WORKOS__CLIENT_ID`,
    /// `SUPABASE__URL`, etc. (double underscore separates nested keys).
    pub fn from_env() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        let cfg = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
}
