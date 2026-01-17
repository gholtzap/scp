use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

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

pub async fn status() -> (StatusCode, Json<StatusResponse>) {
    (
        StatusCode::OK,
        Json(StatusResponse {
            service: "SCP".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub service: String,
    pub version: String,
}
