use std::sync::Arc;
use uuid::Uuid;
use dashmap::DashMap;

#[derive(Clone)]
pub struct AppState {
    pub nf_instance_id: Uuid,
    pub nrf_client: Option<Arc<crate::clients::nrf::NrfClient>>,
    pub http_client: reqwest::Client,
    pub nf_profile_cache: Arc<DashMap<String, CachedNfProfile>>,
}

#[derive(Clone, Debug)]
pub struct CachedNfProfile {
    pub profile: NfProfile,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NfProfile {
    pub nf_instance_id: String,
    pub nf_type: String,
    pub nf_status: String,
    pub ipv4_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
}
