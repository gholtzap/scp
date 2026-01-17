use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use super::problem_details::ProblemDetails;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Gateway timeout: {0}")]
    GatewayTimeout(String),

    #[error("Bad gateway: {0}")]
    BadGateway(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match &self {
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                msg.as_str(),
            ),
            AppError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                msg.as_str(),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                msg.as_str(),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                msg.as_str(),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                msg.as_str(),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                msg.as_str(),
            ),
            AppError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                msg.as_str(),
            ),
            AppError::GatewayTimeout(msg) => (
                StatusCode::GATEWAY_TIMEOUT,
                "Gateway Timeout",
                msg.as_str(),
            ),
            AppError::BadGateway(msg) => (
                StatusCode::BAD_GATEWAY,
                "Bad Gateway",
                msg.as_str(),
            ),
        };

        let problem = ProblemDetails::new(status.as_u16(), title, detail);
        problem.into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::InternalError(format!("Database error: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::GatewayTimeout(err.to_string())
        } else if err.is_connect() {
            AppError::ServiceUnavailable(format!("Failed to connect to upstream service: {}", err))
        } else {
            AppError::BadGateway(err.to_string())
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
