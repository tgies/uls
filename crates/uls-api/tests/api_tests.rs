//! Integration tests for the ULS REST API endpoints.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::NaiveDate;
use tower::ServiceExt;
use uls_api::server::build_router;
use uls_api::ServerConfig;
use uls_core::records::{EntityRecord, HeaderRecord, UlsRecord};
use uls_db::{Database, DatabaseConfig};
use uls_query::QueryEngine;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a test query engine with an in-memory database (empty, initialized).
fn test_engine() -> QueryEngine {
    let config = DatabaseConfig::in_memory();
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();
    QueryEngine::with_database(db)
}

/// Create a test query engine with one sample license pre-loaded.
fn test_engine_with_data() -> QueryEngine {
    let config = DatabaseConfig::in_memory();
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();

    // Insert a test license (HD record)
    let mut header = HeaderRecord::from_fields(&["HD", "12345"]);
    header.unique_system_identifier = 12345;
    header.call_sign = Some("W1TEST".to_string());
    header.license_status = Some('A');
    header.radio_service_code = Some("HA".to_string());
    header.grant_date = Some(NaiveDate::from_ymd_opt(2020, 2, 15).unwrap());
    header.expired_date = Some(NaiveDate::from_ymd_opt(2030, 2, 15).unwrap());
    db.insert_record(&UlsRecord::Header(header)).unwrap();

    // Insert entity (EN record)
    let mut entity = EntityRecord::from_fields(&["EN", "12345"]);
    entity.unique_system_identifier = 12345;
    entity.entity_name = Some("TEST USER".to_string());
    entity.first_name = Some("TEST".to_string());
    entity.last_name = Some("USER".to_string());
    entity.state = Some("CT".to_string());
    entity.city = Some("NEWINGTON".to_string());
    entity.frn = Some("0001234567".to_string());
    db.insert_record(&UlsRecord::Entity(entity)).unwrap();

    QueryEngine::with_database(db)
}

fn server_config() -> ServerConfig {
    ServerConfig::default()
}

/// Helper: read the full response body as bytes.
async fn body_bytes(body: Body) -> Vec<u8> {
    use http_body_util::BodyExt;
    let collected = body.collect().await.unwrap();
    collected.to_bytes().to_vec()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint() {
    let app = build_router(test_engine(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_stats_endpoint() {
    let app = build_router(test_engine(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["total_licenses"], 0);
}

#[tokio::test]
async fn test_lookup_found() {
    let app = build_router(test_engine_with_data(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses/W1TEST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_lookup_not_found() {
    let app = build_router(test_engine(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses/NONEXIST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"], "not_found");
}

#[tokio::test]
async fn test_frn_lookup() {
    let app = build_router(test_engine_with_data(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/frn/0001234567")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["count"], 1);
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_frn_invalid_format() {
    let app = build_router(test_engine(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/frn/abc")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_by_state() {
    let app = build_router(test_engine_with_data(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses?state=CT")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["count"], 1);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_search_empty_results() {
    let app = build_router(test_engine_with_data(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses?state=XX")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn test_search_with_limit() {
    let app = build_router(test_engine_with_data(), &server_config());

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses?state=CT&limit=10&offset=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = body_bytes(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["limit"], 10);
    assert_eq!(json["offset"], 0);
}
