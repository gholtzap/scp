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
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

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
        let profile = clients::nrf::NfProfile {
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
    }

    let app = routes::create_routes(state)
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

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
