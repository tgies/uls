//! Additional integration tests for FccClient covering download branches not
//! exercised by download_integration.rs: content-length handling, incomplete
//! downloads, retry exhaustion, daily-file fan-out, and cached-metadata helpers.

use std::io::Write;
use std::time::Duration;

use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use uls_download::{DownloadConfig, DownloadError, DownloadResult, FccClient, ServiceCatalog};

/// Helper to create a minimal valid ZIP file for tests.
fn create_test_zip(content: &[u8]) -> Vec<u8> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default();
    zip.start_file("HD.dat", options).unwrap();
    zip.write_all(content).unwrap();
    zip.finish().unwrap();
    buffer.into_inner()
}

/// Create a test client pointed at the wiremock server.
fn create_test_client(cache_dir: &std::path::Path, server_uri: &str) -> FccClient {
    let config = DownloadConfig::with_cache_dir(cache_dir.to_path_buf())
        .with_base_url(server_uri)
        .with_timeout(Duration::from_secs(10));
    FccClient::new(config).expect("Failed to create test client")
}

fn noop() -> uls_download::ProgressCallback {
    std::sync::Arc::new(|_| {})
}

// =============================================================================
// Content-Length handling
// =============================================================================

#[tokio::test]
async fn test_download_succeeds_without_content_length() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");

    // No Content-Length header: size cannot be verified, download still succeeds.
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_content.clone()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (path, result) = client.download_file(&file, noop()).await.unwrap();

    assert_eq!(result, DownloadResult::Downloaded);
    assert_eq!(std::fs::read(&path).unwrap(), zip_content);
}

// =============================================================================
// Status-code mapping
// =============================================================================

#[tokio::test]
async fn test_server_error_maps_status_and_url() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    // Disable retries so a single 503 surfaces immediately.
    let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_timeout(Duration::from_secs(10));
    let mut config = config;
    config.max_retries = 0;
    let client = FccClient::new(config).unwrap();

    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let err = client.download_file(&file, noop()).await.unwrap_err();

    match err {
        DownloadError::ServerError { status, url } => {
            assert_eq!(status, 503);
            assert!(url.ends_with("/complete/l_amat.zip"), "url was {url}");
        }
        other => panic!("expected ServerError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_not_found_short_circuits_without_retry() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_clone = hits.clone();

    // 404 must not be retried even though max_retries defaults to 3.
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(move |_: &wiremock::Request| {
            hits_clone.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(404)
        })
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let err = client.download_file(&file, noop()).await.unwrap_err();

    assert!(matches!(err, DownloadError::NotFound { .. }));
    assert_eq!(hits.load(Ordering::SeqCst), 1, "404 should not be retried");
}

// =============================================================================
// Retry behavior with controlled time
// =============================================================================

#[tokio::test]
async fn test_retry_then_success() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_timeout(Duration::from_secs(10));
    let mut config = config;
    config.max_retries = 3;
    config.retry_delay = Duration::from_millis(10);
    let client = FccClient::new(config).unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();
    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");

    // First attempt 500, second attempt succeeds.
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(move |_: &wiremock::Request| {
            let n = count_clone.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                ResponseTemplate::new(500)
            } else {
                ResponseTemplate::new(200)
                    .set_body_bytes(zip_content.clone())
                    .insert_header("Content-Length", zip_content.len().to_string())
            }
        })
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (_, result) = client.download_file(&file, noop()).await.unwrap();

    assert_eq!(result, DownloadResult::Downloaded);
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_retry_exhaustion_returns_last_error() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_timeout(Duration::from_secs(10));
    let mut config = config;
    config.max_retries = 2;
    config.retry_delay = Duration::from_millis(10);
    let client = FccClient::new(config).unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(move |_: &wiremock::Request| {
            count_clone.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(500)
        })
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let err = client.download_file(&file, noop()).await.unwrap_err();

    assert!(matches!(
        err,
        DownloadError::ServerError { status: 500, .. }
    ));
    // Initial attempt plus 2 retries = 3 requests.
    assert_eq!(count.load(Ordering::SeqCst), 3);
}

// =============================================================================
// Daily file fan-out
// =============================================================================

#[tokio::test]
async fn test_download_all_daily_skips_missing_days() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"daily");

    // Monday and Friday are present; the rest 404 and must be skipped.
    for day in ["mon", "fri"] {
        Mock::given(method("GET"))
            .and(path(format!("/daily/l_am_{day}.zip")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(zip_content.clone())
                    .insert_header("Content-Length", zip_content.len().to_string()),
            )
            .mount(&mock_server)
            .await;
    }
    for day in ["sun", "tue", "wed", "thu", "sat"] {
        Mock::given(method("GET"))
            .and(path(format!("/daily/l_am_{day}.zip")))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;
    }

    let paths = client.download_all_daily("amat").await.unwrap();
    assert_eq!(paths.len(), 2);
    let names: Vec<String> = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"l_am_mon.zip".to_string()));
    assert!(names.contains(&"l_am_fri.zip".to_string()));
}

#[tokio::test]
async fn test_download_all_daily_propagates_server_error() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    // No retries so the 500 surfaces from the fan-out promptly.
    let mut config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_timeout(Duration::from_secs(10));
    config.max_retries = 0;
    let client = FccClient::new(config).unwrap();

    // Sunday (first in Weekday::ALL) returns 500.
    Mock::given(method("GET"))
        .and(path("/daily/l_am_sun.zip"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let err = client.download_all_daily("amat").await.unwrap_err();
    assert!(matches!(
        err,
        DownloadError::ServerError { status: 500, .. }
    ));
}

#[tokio::test]
async fn test_download_daily_for_date() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"wednesday");

    // 2026-01-14 is a Wednesday.
    Mock::given(method("GET"))
        .and(path("/daily/l_am_wed.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    let path = client.download_daily_for_date("amat", date).await.unwrap();
    assert_eq!(path.file_name().unwrap().to_str().unwrap(), "l_am_wed.zip");
}

#[tokio::test]
async fn test_download_applications() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"apps");

    Mock::given(method("GET"))
        .and(path("/complete/a_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let path = client.download_applications("amat").await.unwrap();
    assert_eq!(path.file_name().unwrap().to_str().unwrap(), "a_amat.zip");
}

// =============================================================================
// Cached metadata helpers
// =============================================================================

#[tokio::test]
async fn test_get_cached_etag_none_then_some() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let file = ServiceCatalog::complete_license("amat").unwrap();
    // Nothing cached yet.
    assert_eq!(client.get_cached_etag(&file), None);

    let zip_content = create_test_zip(b"data");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", "\"abc-123\""),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    client.download_complete("amat").await.unwrap();
    assert_eq!(
        client.get_cached_etag(&file),
        Some("\"abc-123\"".to_string())
    );
}

#[tokio::test]
async fn test_get_cached_file_date_none_then_some() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let file = ServiceCatalog::complete_license("amat").unwrap();
    assert!(client.get_cached_file_date(&file).is_none());

    let zip_content = create_test_zip(b"data");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    client.download_complete("amat").await.unwrap();
    assert!(client.get_cached_file_date(&file).is_some());
}

#[tokio::test]
async fn test_stale_etag_ignored_when_data_file_missing() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    // Plant a stray .etag without the corresponding data file.
    let etag_path = temp_dir.path().join("l_amat.zip.etag");
    std::fs::write(&etag_path, "\"stale-etag\"").unwrap();

    let head_hits = Arc::new(AtomicUsize::new(0));
    let head_hits_clone = head_hits.clone();
    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(move |_: &wiremock::Request| {
            head_hits_clone.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).insert_header("ETag", "\"stale-etag\"")
        })
        .mount(&mock_server)
        .await;

    let zip_content = create_test_zip(b"fresh");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", "\"stale-etag\""),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (_, result) = client.download_file(&file, noop()).await.unwrap();

    // Missing data file means the cached ETag is ignored: no HEAD probe, full GET.
    assert_eq!(result, DownloadResult::Downloaded);
    assert_eq!(head_hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_changed_head_etag_then_304_returns_not_modified() {
    // Cache file present with matching ETag but HEAD reports a different ETag,
    // forcing a conditional GET that the server answers with 304.
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"v1");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", "\"v1\""),
        )
        .expect(1)
        .mount(&mock_server)
        .await;
    client.download_complete("amat").await.unwrap();
    mock_server.reset().await;

    // HEAD says the remote changed, so a conditional GET fires; server replies 304.
    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"v2\""))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(304))
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (_, result) = client.download_file(&file, noop()).await.unwrap();
    assert_eq!(result, DownloadResult::NotModified);
}

// =============================================================================
// check_for_updates error path
// =============================================================================

#[tokio::test]
async fn test_check_for_updates_server_error() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    let err = client.check_for_updates("amat").await.unwrap_err();
    assert!(matches!(
        err,
        DownloadError::ServerError { status: 500, .. }
    ));
}

#[tokio::test]
async fn test_check_for_updates_unknown_service() {
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), "http://localhost:1");
    let err = client.check_for_updates("not-a-service").await.unwrap_err();
    assert!(matches!(err, DownloadError::UnknownService(_)));
}

// =============================================================================
// Constructor / config interaction
// =============================================================================

#[test]
fn test_new_creates_missing_cache_dir() {
    let temp_dir = TempDir::new().unwrap();
    let nested = temp_dir.path().join("a").join("b").join("cache");
    assert!(!nested.exists());

    let config = DownloadConfig::with_cache_dir(nested.clone());
    let _client = FccClient::new(config).unwrap();
    assert!(nested.exists());
}

#[tokio::test]
async fn test_invalid_user_agent_falls_back_to_default_agent() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    // A header value containing control characters cannot be parsed; the client
    // substitutes the DEFAULT_USER_AGENT fallback. The GET only matches when that
    // exact agent is sent, so a successful download proves the substitution.
    let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_user_agent("bad\nagent")
        .with_timeout(Duration::from_secs(10));
    let client = FccClient::new(config).expect("client construction should not fail");

    let zip_content = create_test_zip(b"data");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .and(header(
            "user-agent",
            uls_download::config::DEFAULT_USER_AGENT,
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (path, result) = client.download_file(&file, noop()).await.unwrap();
    assert_eq!(result, DownloadResult::Downloaded);
    assert_eq!(std::fs::read(&path).unwrap(), zip_content);
}

#[tokio::test]
async fn test_unknown_service_resolution_errors_before_network() {
    let temp_dir = TempDir::new().unwrap();
    // Pointed at an unroutable address: the UnknownService error must precede any
    // attempt to reach it, so the call returns without a network failure.
    let client = create_test_client(temp_dir.path(), "http://localhost:1");

    let err = client.download_complete("zzz").await.unwrap_err();
    assert!(matches!(err, DownloadError::UnknownService(_)));
}
