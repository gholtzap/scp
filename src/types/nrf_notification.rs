use serde::{Deserialize, Serialize};
use super::NfProfile;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationEventType {
    NfRegistered,
    NfDeregistered,
    NfProfileChanged,
    NfStatusChanged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NrfNotification {
    pub event: NotificationEventType,
    pub nf_instance_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_profile: Option<NfProfile>,
}
