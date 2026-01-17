use std::sync::Arc;
use uuid::Uuid;
use dashmap::DashMap;
use super::nf_profile::CachedNfProfile;
use super::retry_config::RetryConfig;
use crate::services::load_balancer::LoadBalancer;

#[derive(Clone)]
pub struct AppState {
    pub nf_instance_id: Uuid,
    pub nrf_client: Option<Arc<crate::clients::nrf::NrfClient>>,
    pub http_client: reqwest::Client,
    pub nf_profile_cache: Arc<DashMap<String, CachedNfProfile>>,
    pub load_balancer: LoadBalancer,
    pub retry_config: RetryConfig,
}
