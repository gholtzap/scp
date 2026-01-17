use mongodb::{Client, Database};
use std::sync::Arc;
use dashmap::DashMap;
use crate::config::Config;
use crate::types::{AppState, CachedNfProfile};

pub async fn init(config: &Config) -> anyhow::Result<AppState> {
    let client = Client::with_uri_str(&config.mongodb_uri).await?;
    let db = client.database("scp");

    tracing::info!("Connected to MongoDB");

    init_collections(&db).await?;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let nrf_client = if let Some(nrf_uri) = &config.nrf_uri {
        Some(Arc::new(
            crate::clients::nrf::NrfClient::new(nrf_uri.clone(), http_client.clone())
        ))
    } else {
        tracing::warn!("NRF URI not configured, service discovery will be unavailable");
        None
    };

    let nf_profile_cache = Arc::new(DashMap::new());

    let nf_instance_id = uuid::Uuid::parse_str(&config.nf_instance_id)?;

    Ok(AppState {
        nf_instance_id,
        nrf_client,
        http_client,
        nf_profile_cache,
    })
}

async fn init_collections(_db: &Database) -> anyhow::Result<()> {
    Ok(())
}
