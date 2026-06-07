//! Integration tests for the ULS REST API endpoints.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::NaiveDate;
use tower::ServiceExt;
use uls_api::server::build_router;
use uls_api::ServerConfig;
use uls_core::records::{AmateurRecord, EntityRecord, HeaderRecord, UlsRecord};
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

/// Create a test query engine with one Extra-class amateur (HA) license, for
/// service- and class-filter tests.
fn test_engine_with_amateur() -> QueryEngine {
    let config = DatabaseConfig::in_memory();
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();

    let mut header = HeaderRecord::from_fields(&["HD", "999"]);
    header.unique_system_identifier = 999;
    header.call_sign = Some("K9EXTRA".to_string());
    header.license_status = Some('A');
    header.radio_service_code = Some("HA".to_string());
    header.grant_date = Some(NaiveDate::from_ymd_opt(2021, 6, 1).unwrap());
    header.expired_date = Some(NaiveDate::from_ymd_opt(2031, 6, 1).unwrap());
    db.insert_record(&UlsRecord::Header(header)).unwrap();

    let mut entity = EntityRecord::from_fields(&["EN", "999"]);
    entity.unique_system_identifier = 999;
    entity.entity_name = Some("EXTRA OP".to_string());
    entity.last_name = Some("OP".to_string());
    entity.state = Some("TX".to_string());
    entity.city = Some("AUSTIN".to_string());
    entity.zip_code = Some("78701".to_string());
    entity.frn = Some("0009998888".to_string());
    db.insert_record(&UlsRecord::Entity(entity)).unwrap();

    let mut amateur = AmateurRecord::from_fields(&["AM", "999"]);
    amateur.unique_system_identifier = 999;
    amateur.callsign = Some("K9EXTRA".to_string());
    amateur.operator_class = Some('E');
    db.insert_record(&UlsRecord::Amateur(amateur)).unwrap();

    QueryEngine::with_database(db)
}

/// Create a test query engine with two rows in different cities/states, for
/// city, filter-expression, and sort tests.
fn test_engine_two_cities() -> QueryEngine {
    let config = DatabaseConfig::in_memory();
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();

    // Row 1: W1TEST in NEWINGTON, CT.
    let mut header1 = HeaderRecord::from_fields(&["HD", "1"]);
    header1.unique_system_identifier = 1;
    header1.call_sign = Some("W1TEST".to_string());
    header1.license_status = Some('A');
    header1.radio_service_code = Some("HA".to_string());
    header1.grant_date = Some(NaiveDate::from_ymd_opt(2020, 2, 15).unwrap());
    db.insert_record(&UlsRecord::Header(header1)).unwrap();

    let mut entity1 = EntityRecord::from_fields(&["EN", "1"]);
    entity1.unique_system_identifier = 1;
    entity1.entity_name = Some("ALPHA USER".to_string());
    entity1.last_name = Some("USER".to_string());
    entity1.state = Some("CT".to_string());
    entity1.city = Some("NEWINGTON".to_string());
    entity1.frn = Some("0001234567".to_string());
    db.insert_record(&UlsRecord::Entity(entity1)).unwrap();

    // Row 2: K2OTHER in BOSTON, MA.
    let mut header2 = HeaderRecord::from_fields(&["HD", "2"]);
    header2.unique_system_identifier = 2;
    header2.call_sign = Some("K2OTHER".to_string());
    header2.license_status = Some('A');
    header2.radio_service_code = Some("HA".to_string());
    header2.grant_date = Some(NaiveDate::from_ymd_opt(2020, 2, 15).unwrap());
    db.insert_record(&UlsRecord::Header(header2)).unwrap();

    let mut entity2 = EntityRecord::from_fields(&["EN", "2"]);
    entity2.unique_system_identifier = 2;
    entity2.entity_name = Some("BRAVO PERSON".to_string());
    entity2.last_name = Some("PERSON".to_string());
    entity2.state = Some("MA".to_string());
    entity2.city = Some("BOSTON".to_string());
    entity2.frn = Some("0007654321".to_string());
    db.insert_record(&UlsRecord::Entity(entity2)).unwrap();

    QueryEngine::with_database(db)
}

/// Build an engine from (usi, call_sign, entity_name, license_status) rows
/// (all HA service, granted 2020-02-15) for discriminating name/status filter
/// tests that need a non-matching row to prove exclusion.
fn engine_with_rows(rows: &[(i64, &str, &str, char)]) -> QueryEngine {
    let config = DatabaseConfig::in_memory();
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();

    for &(usi, call, name, status) in rows {
        let mut header = HeaderRecord::from_fields(&["HD", "0"]);
        header.unique_system_identifier = usi;
        header.call_sign = Some(call.to_string());
        header.license_status = Some(status);
        header.radio_service_code = Some("HA".to_string());
        header.grant_date = Some(NaiveDate::from_ymd_opt(2020, 2, 15).unwrap());
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let mut entity = EntityRecord::from_fields(&["EN", "0"]);
        entity.unique_system_identifier = usi;
        entity.entity_name = Some(name.to_string());
        db.insert_record(&UlsRecord::Entity(entity)).unwrap();
    }

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

/// Helper: GET `uri` against a freshly built router and return (status, json body).
async fn get_json(engine: QueryEngine, uri: &str) -> (StatusCode, serde_json::Value) {
    let app = build_router(engine, &server_config());
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = body_bytes(resp.into_body()).await;
    let json = serde_json::from_slice(&bytes).unwrap();
    (status, json)
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

// ---------------------------------------------------------------------------
// FRN handler branches
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_frn_valid_format_but_no_match_is_404() {
    // Ten digits, well-formed, but no licensee carries this FRN.
    let (status, json) = get_json(test_engine_with_data(), "/frn/9999999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "not_found");
    assert!(json["message"].as_str().unwrap().contains("9999999999"));
}

#[tokio::test]
async fn test_frn_too_short_is_400() {
    let (status, json) = get_json(test_engine(), "/frn/123").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "bad_request");
    assert_eq!(json["message"], "FRN must be exactly 10 digits");
}

#[tokio::test]
async fn test_frn_non_digit_ten_chars_is_400() {
    // Length is right but contains a non-digit, so the digit check rejects it.
    let (status, json) = get_json(test_engine(), "/frn/00012345AB").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "bad_request");
}

#[tokio::test]
async fn test_frn_response_has_zero_pagination_fields() {
    let (status, json) = get_json(test_engine_with_data(), "/frn/0001234567").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["limit"], 0);
    assert_eq!(json["offset"], 0);
    assert_eq!(json["count"], 1);
}

// ---------------------------------------------------------------------------
// Search query-param branches
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_search_unknown_service_is_400() {
    let (status, json) = get_json(test_engine_with_data(), "/licenses?service=walkie").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "bad_request");
    assert!(json["message"].as_str().unwrap().contains("walkie"));
}

#[tokio::test]
async fn test_search_service_amateur_matches_ha() {
    let (status, json) = get_json(test_engine_with_amateur(), "/licenses?service=amateur").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "K9EXTRA");
}

#[tokio::test]
async fn test_search_service_gmrs_excludes_amateur() {
    // The only row is an HA license, so a GMRS (ZA) filter yields nothing.
    let (status, json) = get_json(test_engine_with_amateur(), "/licenses?service=gmrs").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn test_search_by_callsign() {
    let (status, json) = get_json(test_engine_with_data(), "/licenses?callsign=W1TEST").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_search_by_frn_param() {
    let (status, json) = get_json(test_engine_with_data(), "/licenses?frn=0001234567").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["frn"], "0001234567");
}

#[tokio::test]
async fn test_search_by_city() {
    // city=NEWINGTON must match only the Newington row, not the Boston one.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses?city=NEWINGTON").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["city"], "NEWINGTON");
    assert_eq!(json["data"][0]["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_search_no_filter_returns_all_rows() {
    // With no filter, both rows come back.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 2);
}

#[tokio::test]
async fn test_search_by_zip() {
    let (status, json) = get_json(test_engine_with_amateur(), "/licenses?zip=78701").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["zip_code"], "78701");
}

#[tokio::test]
async fn test_search_by_status_active() {
    // Single-char status is uppercased; the sample row is status 'A'.
    let (status, json) = get_json(test_engine_with_data(), "/licenses?status=a").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_search_by_status_no_match() {
    let (status, json) = get_json(test_engine_with_data(), "/licenses?status=X").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn test_search_by_operator_class() {
    let (status, json) = get_json(test_engine_with_amateur(), "/licenses?class=e").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "K9EXTRA");
}

#[tokio::test]
async fn test_search_name_plain_is_wrapped_in_wildcards() {
    // A name without glob chars is wrapped as *NAME* and matches a substring.
    let (status, json) = get_json(test_engine_with_data(), "/licenses?name=USER").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_search_name_with_glob_passed_through() {
    // `name=TEST*` is used as the LIKE pattern as-is (TEST%), matching only names
    // that start with TEST. "CONTEST CLUB" has TEST as an interior substring and
    // must be excluded; a double-wrapped pattern (%TEST%%) would wrongly include it.
    let engine = engine_with_rows(&[
        (1, "W1AAA", "TEST USER", 'A'),
        (2, "W2BBB", "CONTEST CLUB", 'A'),
    ]);
    let (status, json) = get_json(engine, "/licenses?name=TEST*").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "W1AAA");
}

#[tokio::test]
async fn test_search_active_only_flag() {
    // active=true filters to license_status 'A', excluding the expired ('E') row.
    let engine = engine_with_rows(&[
        (1, "W1AAA", "ACTIVE OP", 'A'),
        (2, "W2BBB", "EXPIRED OP", 'E'),
    ]);
    let (status, json) = get_json(engine, "/licenses?active=true").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "W1AAA");
}

#[tokio::test]
async fn test_search_filter_expression() {
    // Two rows differ by state; the filter clause must exclude the MA row.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses?filter=state=CT").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_search_filter_only_empty_segments_is_no_filter() {
    // Filter segments that are only empty/whitespace add no clause, so both
    // rows survive.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses?filter=,,,%20,%20").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 2);
}

#[tokio::test]
async fn test_search_filter_empty_segments_around_valid_clause() {
    // Empty segments are skipped; the surviving state=CT clause excludes the MA row.
    let (status, json) =
        get_json(test_engine_two_cities(), "/licenses?filter=,state=CT,%20,").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["data"][0]["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_search_sort_field_orders_rows() {
    // Ascending sort by call_sign orders K2OTHER before W1TEST.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses?sort=call_sign").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 2);
    assert_eq!(json["data"][0]["call_sign"], "K2OTHER");
    assert_eq!(json["data"][1]["call_sign"], "W1TEST");
}

#[tokio::test]
async fn test_search_sort_field_descending() {
    // A leading '-' flips the order so W1TEST sorts first.
    let (status, json) = get_json(test_engine_two_cities(), "/licenses?sort=-call_sign").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 2);
    assert_eq!(json["data"][0]["call_sign"], "W1TEST");
    assert_eq!(json["data"][1]["call_sign"], "K2OTHER");
}

#[tokio::test]
async fn test_search_limit_clamped_to_max() {
    // Requested limit above MAX_LIMIT (1000) is clamped to 1000 in the response.
    let (status, json) = get_json(test_engine_with_data(), "/licenses?limit=99999").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["limit"], 1000);
}

#[tokio::test]
async fn test_search_default_limit_applied() {
    // No limit param yields the DEFAULT_LIMIT of 50.
    let (status, json) = get_json(test_engine_with_data(), "/licenses?state=CT").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["limit"], 50);
}

#[tokio::test]
async fn test_search_offset_applied() {
    // Offset past the single row yields an empty page but echoes the offset.
    let (status, json) = get_json(test_engine_with_data(), "/licenses?state=CT&offset=5").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["offset"], 5);
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn test_search_bad_limit_type_is_400() {
    // A non-numeric limit fails Query extraction before the handler runs.
    let app = build_router(test_engine_with_data(), &server_config());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/licenses?limit=notanumber")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_granted_after_filter() {
    // Sample grant date is 2020-02-15, so an earlier bound keeps the row.
    let (status, json) = get_json(
        test_engine_with_data(),
        "/licenses?granted_after=2019-01-01",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_search_granted_after_excludes_older() {
    let (status, json) = get_json(
        test_engine_with_data(),
        "/licenses?granted_after=2025-01-01",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
}

// ---------------------------------------------------------------------------
// Stats handler with data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_stats_counts_loaded_license() {
    let (status, json) = get_json(test_engine_with_data(), "/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total_licenses"], 1);
}

// ---------------------------------------------------------------------------
// Router wiring / ServerConfig (server.rs)
// ---------------------------------------------------------------------------

#[test]
fn server_config_default_values() {
    let config = ServerConfig::default();
    assert_eq!(config.bind, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert!(config.cors_origins.is_empty());
}

#[tokio::test]
async fn test_unknown_route_is_404() {
    let app = build_router(test_engine(), &server_config());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_known_routes_are_wired() {
    // Each declared route returns 200 on the pre-loaded engine, not a 404.
    for uri in [
        "/health",
        "/stats",
        "/licenses",
        "/licenses/W1TEST",
        "/frn/0001234567",
    ] {
        let app = build_router(test_engine_with_data(), &server_config());
        let resp = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "route {uri} should be wired");
    }
}

#[tokio::test]
async fn test_cors_wildcard_layer_adds_allow_origin_header() {
    let config = ServerConfig {
        cors_origins: vec!["*".to_string()],
        ..ServerConfig::default()
    };
    let app = build_router(test_engine(), &config);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "https://anything.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("*")
    );
}

#[tokio::test]
async fn test_cors_specific_origin_layer_echoes_allowed_origin() {
    let config = ServerConfig {
        cors_origins: vec!["https://allowed.test".to_string()],
        ..ServerConfig::default()
    };
    let app = build_router(test_engine(), &config);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "https://allowed.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("https://allowed.test")
    );
}

#[tokio::test]
async fn test_no_cors_layer_when_origins_empty() {
    let config = ServerConfig::default();
    let app = build_router(test_engine(), &config);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "https://anything.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}
