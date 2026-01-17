use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug)]
pub struct CachedNfProfile {
    pub profile: NfProfile,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}
