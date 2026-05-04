//! Bundu Identity gateway — binary entry point.

use std::sync::Arc;

use bundu_identity_server::{build_router, AppConfig, AppState};
use bundu_jwt::PlatformJwtSigner;
use bundu_service_bus::CloudflareQueuesPublisher;
use bundu_supabase::HttpIdentityResolver;
use bundu_workos::HttpWorkOsClient;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;

    init_tracing(&config.log_level)?;

    tracing::info!(
        environment = %config.environment,
        instance = %config.instance,
        listen = %config.listen_addr,
        "bundu-identity starting"
    );

    let workos = HttpWorkOsClient::new(
        &config.workos.client_id,
        &config.workos.api_key,
    )?;

    let identity = HttpIdentityResolver::new(
        &config.supabase.url,
        &config.supabase.service_role_key,
    )?;

    let jwt = PlatformJwtSigner::new_hs256(&config.jwt.signing_secret)?;

    let bus = CloudflareQueuesPublisher::new(
        &config.cloudflare.account_id,
        &config.cloudflare.queues_api_token,
    )?;

    let state = Arc::new(AppState {
        config: config.clone(),
        workos: Arc::new(workos),
        identity: Arc::new(identity),
        jwt,
        bus: Arc::new(bus),
    });

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing(level: &str) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .try_init()?;
    Ok(())
}
