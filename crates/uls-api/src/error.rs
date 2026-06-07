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

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    /// Read a response body as parsed JSON.
    async fn json_body(resp: Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn not_found_maps_to_404_with_message() {
        let resp = ApiError::NotFound("missing W1XYZ".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = json_body(resp).await;
        assert_eq!(body["error"], "not_found");
        assert_eq!(body["message"], "missing W1XYZ");
    }

    #[tokio::test]
    async fn bad_request_maps_to_400_with_message() {
        let resp = ApiError::BadRequest("bad input".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = json_body(resp).await;
        assert_eq!(body["error"], "bad_request");
        assert_eq!(body["message"], "bad input");
    }

    #[tokio::test]
    async fn not_initialized_maps_to_503_with_static_message() {
        let resp = ApiError::NotInitialized.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = json_body(resp).await;
        assert_eq!(body["error"], "not_initialized");
        assert_eq!(
            body["message"],
            "Database not initialized. Run 'uls update' first."
        );
    }

    #[tokio::test]
    async fn internal_maps_to_500_with_message() {
        let resp = ApiError::Internal("boom".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = json_body(resp).await;
        assert_eq!(body["error"], "internal_error");
        assert_eq!(body["message"], "boom");
    }

    #[test]
    fn query_error_not_initialized_converts_to_api_not_initialized() {
        let api: ApiError = uls_query::QueryError::NotInitialized.into();
        assert!(matches!(api, ApiError::NotInitialized));
    }

    #[test]
    fn query_error_other_converts_to_internal_preserving_message() {
        // A query against an in-memory database with no schema returns a
        // non-NotInitialized QueryError (a SQLite "no such table" error),
        // which the conversion maps to ApiError::Internal.
        let db = uls_db::Database::with_config(uls_db::DatabaseConfig::in_memory()).unwrap();
        let engine = uls_query::QueryEngine::with_database(db);
        let err = engine
            .lookup("W1AW")
            .expect_err("lookup on uninitialized schema should error");
        assert!(!matches!(err, uls_query::QueryError::NotInitialized));

        let display = err.to_string();
        let api: ApiError = err.into();
        match api {
            ApiError::Internal(msg) => assert_eq!(msg, display),
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    #[test]
    fn error_display_includes_variant_context() {
        assert_eq!(
            ApiError::NotFound("x".to_string()).to_string(),
            "not found: x"
        );
        assert_eq!(
            ApiError::BadRequest("y".to_string()).to_string(),
            "bad request: y"
        );
        assert_eq!(
            ApiError::NotInitialized.to_string(),
            "database not initialized"
        );
        assert_eq!(
            ApiError::Internal("z".to_string()).to_string(),
            "internal error: z"
        );
    }
}
