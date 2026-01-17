use axum::{routing::{any, get}, Router};
use crate::{handlers, types::AppState};

pub fn create_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/status", get(handlers::health::status))
        .fallback(handlers::proxy::proxy_request)
        .with_state(app_state)
}
