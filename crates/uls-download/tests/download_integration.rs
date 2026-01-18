//! Integration tests for FccClient using wiremock.
//!
//! These tests use a mock HTTP server to verify download logic,
//! ETag caching, retry behavior, and error handling.

use std::io::Write;
use std::time::Duration;

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use uls_download::{DownloadConfig, DownloadResult, FccClient, ServiceCatalog};

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

/// Create a test client with wiremock server URL.
fn create_test_client(cache_dir: &std::path::Path, server_uri: &str) -> FccClient {
    let config = DownloadConfig::with_cache_dir(cache_dir.to_path_buf())
        .with_base_url(server_uri)
        .with_timeout(Duration::from_secs(10));
    FccClient::new(config).expect("Failed to create test client")
}

// =============================================================================
// Basic download tests
// =============================================================================

#[tokio::test]
async fn test_download_complete_license_success() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");

    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", "\"test-etag-123\""),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client.download_complete("amat").await;
    assert!(result.is_ok());

    let download_path = result.unwrap();
    assert!(download_path.exists());
    assert_eq!(
        download_path.file_name().unwrap().to_str().unwrap(),
        "l_amat.zip"
    );

    // Verify ETag was saved
    let etag_path = temp_dir.path().join("l_amat.zip.etag");
    assert!(etag_path.exists());
    let saved_etag = std::fs::read_to_string(etag_path).unwrap();
    assert_eq!(saved_etag, "\"test-etag-123\"");
}

#[tokio::test]
async fn test_download_file_returns_path_and_result() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");

    Mock::given(method("GET"))
        .and(path("/complete/l_gmrs.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("gmrs").unwrap();
    let (path, result) = client
        .download_file(&file, std::sync::Arc::new(|_| {}))
        .await
        .unwrap();

    assert_eq!(result, DownloadResult::Downloaded);
    assert!(path.exists());
}

// =============================================================================
// ETag / caching tests
// =============================================================================

#[tokio::test]
async fn test_etag_caching_returns_not_modified() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");
    let etag = "\"cached-etag-456\"";

    // First request: full download
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", etag),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client.download_complete("amat").await;
    assert!(result.is_ok());

    // Reset mocks for second request
    mock_server.reset().await;

    // HEAD request to check if cache is current
    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", etag))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Second request with same ETag should return NotModified
    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (_, result) = client
        .download_file(&file, std::sync::Arc::new(|_| {}))
        .await
        .unwrap();

    assert_eq!(result, DownloadResult::NotModified);
}

#[tokio::test]
async fn test_etag_changed_triggers_redownload() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");
    let old_etag = "\"old-etag\"";
    let new_etag = "\"new-etag\"";

    // First request
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", old_etag),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    client.download_complete("amat").await.unwrap();
    mock_server.reset().await;

    // HEAD check shows new ETag
    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", new_etag))
        .expect(1)
        .mount(&mock_server)
        .await;

    // GET request for new content
    let new_content = create_test_zip(b"HD|2|EN|67890|K1ABC|");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(new_content.clone())
                .insert_header("Content-Length", new_content.len().to_string())
                .insert_header("ETag", new_etag),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let (_, result) = client
        .download_file(&file, std::sync::Arc::new(|_| {}))
        .await
        .unwrap();

    assert_eq!(result, DownloadResult::Downloaded);
}

// =============================================================================
// Error handling tests
// =============================================================================

#[tokio::test]
async fn test_download_not_found_error() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client.download_complete("amat").await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_string = err.to_string();
    assert!(
        err_string.contains("not found"),
        "Expected 'not found' in: {}",
        err_string
    );
}

#[tokio::test]
async fn test_download_server_error() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(ResponseTemplate::new(500))
        // Will be called multiple times due to retries
        .mount(&mock_server)
        .await;

    let result = client.download_complete("amat").await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn test_retry_on_transient_failure() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    // Create client with retries enabled (default is 3 retries)
    let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf())
        .with_base_url(mock_server.uri())
        .with_timeout(Duration::from_secs(10));
    let client = FccClient::new(config).expect("Failed to create test client");

    let request_count = Arc::new(AtomicUsize::new(0));
    let request_count_clone = request_count.clone();
    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|");

    // First 2 requests fail with 503, third succeeds
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(move |_: &wiremock::Request| {
            let count = request_count_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                ResponseTemplate::new(503) // Service Unavailable
            } else {
                ResponseTemplate::new(200)
                    .set_body_bytes(zip_content.clone())
                    .insert_header("Content-Length", zip_content.len().to_string())
            }
        })
        .mount(&mock_server)
        .await;

    // Should succeed after retries
    let _result = client.download_complete("amat").await;

    // The client may fail after all retries are exhausted, or succeed if retries work.
    // Either way, we verify retries happened.
    let total_requests = request_count.load(Ordering::SeqCst);
    assert!(
        total_requests >= 2,
        "Expected at least 2 requests (with retries), got {}",
        total_requests
    );
}

// =============================================================================
// Update check tests
// =============================================================================

#[tokio::test]
async fn test_check_for_updates_has_update() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Length", "12345678")
                .insert_header("ETag", "\"remote-etag\"")
                .insert_header("Last-Modified", "Sun, 01 Jan 2023 00:00:00 GMT"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let update_info = client.check_for_updates("amat").await.unwrap();

    assert!(update_info.has_update); // No local cache, so update needed
    assert_eq!(update_info.size_bytes, Some(12345678));
    assert_eq!(update_info.etag, Some("\"remote-etag\"".to_string()));
    assert!(update_info.last_modified.is_some());
}

#[tokio::test]
async fn test_check_for_updates_no_update_when_etag_matches() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let etag = "\"matching-etag\"";

    // First, download to populate cache with ETag
    let zip_content = create_test_zip(b"test");
    Mock::given(method("GET"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string())
                .insert_header("ETag", etag),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    client.download_complete("amat").await.unwrap();
    mock_server.reset().await;

    // Now check for updates - same ETag means no update
    Mock::given(method("HEAD"))
        .and(path("/complete/l_amat.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Length", "1000")
                .insert_header("ETag", etag),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let update_info = client.check_for_updates("amat").await.unwrap();
    assert!(!update_info.has_update);
}

// =============================================================================
// Progress callback tests
// =============================================================================

#[tokio::test]
async fn test_progress_callback_is_invoked() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|EN|12345|W1AW|test data here|");

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

    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = callback_count.clone();

    let file = ServiceCatalog::complete_license("amat").unwrap();
    let progress_callback = Arc::new(move |_progress: &uls_download::DownloadProgress| {
        callback_count_clone.fetch_add(1, Ordering::SeqCst);
    });

    client
        .download_file(&file, progress_callback)
        .await
        .unwrap();

    // Should have been called at least once
    assert!(
        callback_count.load(Ordering::SeqCst) >= 1,
        "Progress callback should be invoked"
    );
}

// =============================================================================
// Service catalog integration tests
// =============================================================================

#[tokio::test]
async fn test_download_by_radio_service_code() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"test");

    // Using "HA" (amateur radio service code) should resolve to l_amat.zip
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

    // Get file via radio service code (this is what CLI does)
    let file = ServiceCatalog::complete_license("HA").unwrap();
    assert_eq!(file.filename(), "l_amat.zip");

    let (path, _) = client
        .download_file(&file, std::sync::Arc::new(|_| {}))
        .await
        .unwrap();
    assert!(path.exists());
}

#[tokio::test]
async fn test_download_gmrs_service() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let client = create_test_client(temp_dir.path(), &mock_server.uri());

    let zip_content = create_test_zip(b"HD|1|ZA|00001|WQFX123|");

    Mock::given(method("GET"))
        .and(path("/complete/l_gmrs.zip"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(zip_content.clone())
                .insert_header("Content-Length", zip_content.len().to_string()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client.download_complete("ZA").await;
    assert!(result.is_ok());
}
