use axum::{http::StatusCode, Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::types::AppState;
use crate::services::load_balancer::LoadBalancerStats;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
        }),
    )
}

pub async fn status(State(state): State<AppState>) -> (StatusCode, Json<StatusResponse>) {
    let uptime_secs = state.start_time.elapsed().as_secs();

    let nrf_status = if state.nrf_client.is_some() {
        "connected"
    } else {
        "not_configured"
    };

    let cached_profiles = state.nf_profile_cache.len();

    let lb_stats = state.load_balancer.get_statistics();

    (
        StatusCode::OK,
        Json(StatusResponse {
            service: "SCP".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            nf_instance_id: state.nf_instance_id.to_string(),
            uptime_seconds: uptime_secs,
            nrf_status: nrf_status.to_string(),
            cache: CacheStats {
                cached_nf_profiles: cached_profiles,
            },
            load_balancer: lb_stats,
        }),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub service: String,
    pub version: String,
    pub nf_instance_id: String,
    pub uptime_seconds: u64,
    pub nrf_status: String,
    pub cache: CacheStats,
    pub load_balancer: LoadBalancerStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub cached_nf_profiles: usize,
}
