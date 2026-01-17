use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::types::{AppState, AppError};

pub async fn proxy_request(
    State(_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "message": "Proxy functionality not yet implemented"
        })),
    ))
}
