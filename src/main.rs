mod config;
mod db;
mod handlers;
mod clients;
mod services;
mod types;
mod utils;
mod middleware;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;
use std::time::Duration;

async fn heartbeat_task(
    nrf_client: Arc<clients::nrf::NrfClient>,
    profile: types::NfProfile,
    interval_seconds: u64,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    interval.tick().await;

    loop {
        interval.tick().await;

        match nrf_client.heartbeat(&profile).await {
            Ok(_) => tracing::debug!("Heartbeat sent to NRF"),
            Err(e) => tracing::warn!("Failed to send heartbeat to NRF: {}", e),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scp=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let state = db::init(&config).await?;

    if let Some(ref nrf_client) = state.nrf_client {
        let profile = types::NfProfile {
            nf_instance_id: state.nf_instance_id.to_string(),
            nf_type: "SCP".to_string(),
            nf_status: "REGISTERED".to_string(),
            ipv4_addresses: vec![config.scp_host.clone()],
            fqdn: None,
            capacity: Some(100),
            priority: Some(1),
        };

        match nrf_client.register(&profile).await {
            Ok(_) => tracing::info!("Successfully registered with NRF"),
            Err(e) => tracing::error!("Failed to register with NRF: {}", e),
        }

        let heartbeat_client = nrf_client.clone();
        let heartbeat_profile = profile.clone();
        let heartbeat_interval = config.heartbeat_interval_seconds;
        tokio::spawn(async move {
            heartbeat_task(heartbeat_client, heartbeat_profile, heartbeat_interval).await;
        });
    }

    let app = routes::create_routes(state)
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive())
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    tracing::info!("SCP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
}
