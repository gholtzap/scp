use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use crate::clients::nrf::NfDiscoveryParams;
use crate::types::{AppError, AppState};

pub async fn proxy_request(
    State(state): State<AppState>,
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

    let producer_uri = select_producer(&state, &target_nf_type).await?;

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

    let mut request_builder = state
        .http_client
        .request(method.clone(), &target_url);

    for (key, value) in headers.iter() {
        if !is_hop_by_hop_header(key.as_str()) {
            request_builder = request_builder.header(key, value);
        }
    }

    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes);
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("Failed to forward request: {}", e)))?;

    let status = response.status();
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

async fn select_producer(state: &AppState, target_nf_type: &str) -> Result<String, AppError> {
    let cache_key = format!("nf_type_{}", target_nf_type);

    if let Some(cached) = state.nf_profile_cache.get(&cache_key) {
        let cache_age = chrono::Utc::now() - cached.cached_at;
        if cache_age.num_seconds() < 300 {
            tracing::debug!("Using cached NF profile for {}", target_nf_type);
            return build_producer_uri(&cached.profile);
        }
    }

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

    let selected = instances[0].clone();

    state.nf_profile_cache.insert(
        cache_key,
        crate::types::CachedNfProfile {
            profile: selected.clone(),
            cached_at: chrono::Utc::now(),
        },
    );

    build_producer_uri(&selected)
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
