use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::types::{AppState, NrfNotification, NotificationEventType, CachedNfProfile};

pub async fn handle_nrf_notification(
    State(state): State<AppState>,
    Json(notification): Json<NrfNotification>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Received NRF notification: {:?}", notification.event);

    let nf_instance_id = extract_nf_instance_id(&notification.nf_instance_uri);

    match notification.event {
        NotificationEventType::NfRegistered | NotificationEventType::NfProfileChanged => {
            if let Some(profile) = notification.nf_profile {
                tracing::info!("Updating cache for NF instance: {}", nf_instance_id);

                let cached_profile = CachedNfProfile {
                    profile,
                    cached_at: chrono::Utc::now(),
                };

                state.nf_profile_cache.insert(nf_instance_id.clone(), cached_profile);
            } else {
                tracing::warn!("Received {} event without NF profile",
                    match notification.event {
                        NotificationEventType::NfRegistered => "NfRegistered",
                        NotificationEventType::NfProfileChanged => "NfProfileChanged",
                        _ => "unknown"
                    }
                );
            }
        }
        NotificationEventType::NfDeregistered => {
            tracing::info!("Removing NF instance from cache: {}", nf_instance_id);
            state.nf_profile_cache.remove(&nf_instance_id);
        }
        NotificationEventType::NfStatusChanged => {
            if let Some(profile) = notification.nf_profile {
                tracing::info!("Updating NF status for instance: {} to {}",
                    nf_instance_id, profile.nf_status);

                let cached_profile = CachedNfProfile {
                    profile,
                    cached_at: chrono::Utc::now(),
                };

                state.nf_profile_cache.insert(nf_instance_id.clone(), cached_profile);
            } else {
                tracing::warn!("Received NfStatusChanged event without NF profile");
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

fn extract_nf_instance_id(nf_instance_uri: &str) -> String {
    nf_instance_uri
        .split('/')
        .last()
        .unwrap_or(nf_instance_uri)
        .to_string()
}
