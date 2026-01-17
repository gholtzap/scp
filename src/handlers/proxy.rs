use axum::{
    body::Body,
    extract::{Request, State, ConnectInfo},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use crate::clients::nrf::NfDiscoveryParams;
use crate::types::{AppError, AppState};
use crate::utils::retry_with_backoff;

pub async fn proxy_request(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, AppError> {
    let path = uri.path();
    let query = uri.query();

    tracing::debug!("Proxying {} request to {}", method, path);

    let target_nf_type = extract_nf_type_from_path(path).ok_or_else(|| {
        AppError::BadRequest(format!("Unable to determine target NF type from path: {}", path))
    })?;

    tracing::debug!("Extracted target NF type: {}", target_nf_type);

    let session_id = addr.ip().to_string();

    let (producer_uri, selected_instance_id, _connection_guard) = select_producer(&state, &target_nf_type, &session_id).await?;

    tracing::info!(
        "Forwarding {} {} to producer at {}",
        method,
        path,
        producer_uri
    );

    let target_url = if let Some(q) = query {
        format!("{}{}?{}", producer_uri, path, q)
    } else {
        format!("{}{}", producer_uri, path)
    };

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    let retry_result = retry_with_backoff(&state.retry_config, || {
        let state = state.clone();
        let target_url = target_url.clone();
        let method = method.clone();
        let headers = headers.clone();
        let body_bytes = body_bytes.clone();
        let selected_instance_id = selected_instance_id.clone();

        async move {
            let mut request_builder = state
                .http_client
                .request(method, &target_url);

            for (key, value) in headers.iter() {
                if !is_hop_by_hop_header(key.as_str()) {
                    request_builder = request_builder.header(key, value);
                }
            }

            if !body_bytes.is_empty() {
                request_builder = request_builder.body(body_bytes);
            }

            let response = request_builder.send().await.map_err(|e| {
                AppError::ServiceUnavailable(format!("Request failed: {}", e))
            })?;

            let status = response.status();

            if status.is_server_error() || status == StatusCode::SERVICE_UNAVAILABLE {
                tracing::warn!(
                    "Producer {} returned error status {}, will retry",
                    selected_instance_id,
                    status
                );
                return Err(AppError::ServiceUnavailable(format!(
                    "Producer returned error status: {}",
                    status
                )));
            }

            Ok(response)
        }
    })
    .await;

    match retry_result {
        Ok(response) => {
            let status = response.status();
            state.load_balancer.mark_success(&selected_instance_id);

            let response_headers = response.headers().clone();
            let response_body = response
                .bytes()
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to read response body: {}", e)))?;

            let mut builder = Response::builder().status(status);

            for (key, value) in response_headers.iter() {
                if !is_hop_by_hop_header(key.as_str()) {
                    builder = builder.header(key, value);
                }
            }

            let response = builder
                .body(Body::from(response_body))
                .map_err(|e| AppError::InternalError(format!("Failed to build response: {}", e)))?;

            Ok(response)
        }
        Err(e) => {
            state.load_balancer.mark_failure(&selected_instance_id);
            tracing::error!(
                "Failed to forward request to producer {} after retries: {}",
                selected_instance_id,
                e
            );
            Err(e)
        }
    }
}

fn extract_nf_type_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 2 {
        return None;
    }

    let service_name = parts[1];

    if !service_name.starts_with('n') {
        return None;
    }

    let nf_type = service_name
        .split('-')
        .next()?
        .strip_prefix('n')?
        .to_uppercase();

    Some(nf_type)
}

async fn select_producer(
    state: &AppState,
    target_nf_type: &str,
    session_id: &str,
) -> Result<(String, String, crate::services::load_balancer::ConnectionGuard), AppError> {
    tracing::debug!("Cache miss or expired for {}, querying NRF", target_nf_type);

    let nrf_client = state
        .nrf_client
        .as_ref()
        .ok_or_else(|| AppError::InternalError("NRF client not configured".to_string()))?;

    let params = NfDiscoveryParams {
        target_nf_type: target_nf_type.to_string(),
        requester_nf_type: Some("SCP".to_string()),
        service_names: None,
    };

    let instances: Vec<crate::types::NfProfile> = nrf_client
        .discover(&params)
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("NRF discovery failed: {}", e)))?;

    if instances.is_empty() {
        return Err(AppError::ServiceUnavailable(format!(
            "No available instances found for NF type: {}",
            target_nf_type
        )));
    }

    let selected = state
        .load_balancer
        .select_with_sticky_session(session_id, target_nf_type, &instances)
        .clone();

    let uri = build_producer_uri(&selected)?;
    let instance_id = selected.nf_instance_id.clone();
    let guard = state
        .load_balancer
        .acquire_connection(instance_id.clone());

    Ok((uri, instance_id, guard))
}

fn build_producer_uri(profile: &crate::types::NfProfile) -> Result<String, AppError> {
    if let Some(fqdn) = &profile.fqdn {
        Ok(format!("http://{}", fqdn))
    } else if let Some(ip) = profile.ipv4_addresses.first() {
        Ok(format!("http://{}", ip))
    } else {
        Err(AppError::InternalError(format!(
            "No valid address found for NF instance {}",
            profile.nf_instance_id
        )))
    }
}

fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}
