//! API error types and JSON error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// JSON error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// API-level error type.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("database not initialized")]
    NotInitialized,

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<uls_query::QueryError> for ApiError {
    fn from(err: uls_query::QueryError) -> Self {
        match err {
            uls_query::QueryError::NotInitialized => ApiError::NotInitialized,
            other => ApiError::Internal(other.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_key, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            ApiError::NotInitialized => (
                StatusCode::SERVICE_UNAVAILABLE,
                "not_initialized",
                "Database not initialized. Run 'uls update' first.".to_string(),
            ),
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
            ),
        };

        let body = ErrorResponse {
            error: error_key.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}
