//! FCC data download client.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{NaiveDate, Utc};
use reqwest::header::{HeaderMap, CONTENT_LENGTH, ETAG, IF_NONE_MATCH, LAST_MODIFIED, USER_AGENT};
use reqwest::StatusCode;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::catalog::{DataFile, ServiceCatalog};
use crate::config::DownloadConfig;
use crate::error::{DownloadError, Result};
use crate::progress::{no_progress, DownloadProgress, ProgressCallback};

/// HTTP client for downloading FCC ULS data files.
pub struct FccClient {
    http: reqwest::Client,
    config: DownloadConfig,
}

impl FccClient {
    /// Create a new FCC download client.
    pub fn new(config: DownloadConfig) -> Result<Self> {
        // Ensure cache directory exists
        fs::create_dir_all(&config.cache_dir).map_err(|_| DownloadError::CacheDirectoryError {
            path: config.cache_dir.clone(),
        })?;

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            config
                .user_agent
                .parse()
                .unwrap_or_else(|_| "uls-cli/0.1.0".parse().unwrap()),
        );

        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()?;

        Ok(Self { http, config })
    }

    /// Create a client with default configuration.
    pub fn default_client() -> Result<Self> {
        Self::new(DownloadConfig::default())
    }

    /// Get the full URL for a data file.
    fn url(&self, file: &DataFile) -> String {
        format!("{}/{}", self.config.base_url, file.url_path())
    }

    /// Get the cache path for a data file.
    fn cache_path(&self, file: &DataFile) -> PathBuf {
        self.config.cache_dir.join(file.filename())
    }

    /// Download the complete (weekly) license file for a service.
    pub async fn download_complete(&self, service: &str) -> Result<PathBuf> {
        self.download_complete_with_progress(service, no_progress())
            .await
    }

    /// Download the complete license file with progress callback.
    pub async fn download_complete_with_progress(
        &self,
        service: &str,
        progress: ProgressCallback,
    ) -> Result<PathBuf> {
        let file = ServiceCatalog::complete_license(service)?;
        let (path, _) = self.download_file(&file, progress).await?;
        Ok(path)
    }

    /// Download the complete (weekly) application file for a service.
    pub async fn download_applications(&self, service: &str) -> Result<PathBuf> {
        let file = ServiceCatalog::complete_application(service)?;
        let (path, _) = self.download_file(&file, no_progress()).await?;
        Ok(path)
    }

    /// Download all daily license files for a service.
    pub async fn download_all_daily(&self, service: &str) -> Result<Vec<PathBuf>> {
        let files = ServiceCatalog::daily_licenses(service)?;
        let mut paths = Vec::new();

        for file in files {
            match self.download_file(&file, no_progress()).await {
                Ok((path, _)) => paths.push(path),
                Err(DownloadError::NotFound { .. }) => {
                    debug!("Daily file not available: {}", file.filename());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(paths)
    }

    /// Download the daily license file for a specific date.
    pub async fn download_daily_for_date(&self, service: &str, date: NaiveDate) -> Result<PathBuf> {
        let file = ServiceCatalog::daily_license_for_date(service, date)?;
        let (path, _) = self.download_file(&file, no_progress()).await?;
        Ok(path)
    }

    /// Download a specific data file.
    /// Returns the path and whether the file was newly downloaded or from cache.
    pub async fn download_file(
        &self,
        file: &DataFile,
        progress: ProgressCallback,
    ) -> Result<(PathBuf, DownloadResult)> {
        let url = self.url(file);
        let dest_path = self.cache_path(file);

        // Check for existing cached file and get its ETag
        // Only use cached ETag if the data file actually exists
        let cache_exists = dest_path.exists();
        let cached_etag = if cache_exists {
            self.get_cached_etag(file)
        } else {
            None // Ignore stale ETag if data file is missing
        };

        // If we have a cached file with an ETag, do a HEAD request to check if it's still current
        if cache_exists {
            if let Some(ref local_etag) = cached_etag {
                info!("Checking if cached {} is current...", file.filename());

                // HEAD request to get remote ETag without downloading
                let head_response = self.http.head(&url).send().await?;

                if head_response.status().is_success() {
                    let remote_etag = head_response
                        .headers()
                        .get(ETAG)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    if let Some(ref remote) = remote_etag {
                        if remote == local_etag {
                            info!("Cache is current (ETag match): {}", file.filename());
                            return Ok((dest_path, DownloadResult::NotModified));
                        } else {
                            info!("Remote ETag differs, downloading new version");
                        }
                    }
                }
            }
        }

        info!("Downloading {} to {}", url, dest_path.display());

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                warn!("Retry attempt {} for {}", attempt, file.filename());
                tokio::time::sleep(self.config.retry_delay).await;
            }

            match self
                .do_download(&url, &dest_path, cached_etag.as_deref(), progress.clone())
                .await
            {
                Ok(DownloadResult::Downloaded) => {
                    info!("Downloaded: {}", file.filename());
                    return Ok((dest_path, DownloadResult::Downloaded));
                }
                Ok(DownloadResult::NotModified) => {
                    info!("Not modified (using cache): {}", file.filename());
                    return Ok((dest_path, DownloadResult::NotModified));
                }
                Err(DownloadError::NotFound { .. }) => {
                    return Err(DownloadError::NotFound { url });
                }
                Err(e) if attempt == self.config.max_retries => {
                    return Err(e);
                }
                Err(e) => {
                    warn!("Download attempt failed: {}", e);
                }
            }
        }

        unreachable!()
    }

    /// Perform the actual download.
    async fn do_download(
        &self,
        url: &str,
        dest_path: &Path,
        etag: Option<&str>,
        progress: ProgressCallback,
    ) -> Result<DownloadResult> {
        let mut request = self.http.get(url);

        // Add If-None-Match header if we have a cached ETag
        if let Some(etag) = etag {
            request = request.header(IF_NONE_MATCH, etag);
        }

        let response = request.send().await?;
        let status = response.status();

        match status {
            StatusCode::OK => {
                let total_bytes = response
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok());

                let new_etag = response
                    .headers()
                    .get(ETAG)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let last_modified = response
                    .headers()
                    .get(LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Download with progress tracking
                let mut file = tokio::fs::File::create(dest_path).await?;
                let filename = dest_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let mut prog = DownloadProgress::new(&filename, total_bytes);
                let mut stream = response.bytes_stream();
                let start_time = Instant::now();

                use futures_util::StreamExt;
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;

                    prog.downloaded_bytes += chunk.len() as u64;

                    // Calculate speed
                    let elapsed = start_time.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        prog.speed_bps = (prog.downloaded_bytes as f64 / elapsed) as u64;

                        // Calculate ETA
                        if let Some(total) = total_bytes {
                            let remaining = total.saturating_sub(prog.downloaded_bytes);
                            prog.eta_seconds =
                                Some((remaining as f64 / prog.speed_bps.max(1) as f64) as u64);
                        }
                    }

                    progress(&prog);
                }

                file.flush().await?;

                // Verify download size
                if let Some(expected) = total_bytes {
                    if prog.downloaded_bytes != expected {
                        return Err(DownloadError::IncompleteDownload {
                            expected,
                            actual: prog.downloaded_bytes,
                        });
                    }
                }

                // Save metadata (ETag for future conditional requests)
                if let Some(etag) = new_etag {
                    self.save_etag(dest_path, &etag)?;
                }

                if let Some(modified) = last_modified {
                    debug!("Last-Modified: {}", modified);
                }

                Ok(DownloadResult::Downloaded)
            }
            StatusCode::NOT_MODIFIED => Ok(DownloadResult::NotModified),
            StatusCode::NOT_FOUND => Err(DownloadError::NotFound {
                url: url.to_string(),
            }),
            _ => Err(DownloadError::ServerError {
                status: status.as_u16(),
                url: url.to_string(),
            }),
        }
    }

    /// Check if FCC has updates available for a service.
    pub async fn check_for_updates(&self, service: &str) -> Result<UpdateInfo> {
        let file = ServiceCatalog::complete_license(service)?;
        let url = self.url(&file);

        let response = self.http.head(&url).send().await?;

        if !response.status().is_success() {
            return Err(DownloadError::ServerError {
                status: response.status().as_u16(),
                url,
            });
        }

        let content_length = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let cached_etag = self.get_cached_etag(&file);
        let has_update = match (&etag, &cached_etag) {
            (Some(remote), Some(local)) => remote != local,
            _ => true, // Assume update needed if we can't compare
        };

        Ok(UpdateInfo {
            file,
            size_bytes: content_length,
            last_modified,
            etag,
            has_update,
            checked_at: Utc::now(),
        })
    }

    /// Get the cached ETag for a file.
    pub fn get_cached_etag(&self, file: &DataFile) -> Option<String> {
        let etag_path = self
            .config
            .cache_dir
            .join(format!("{}.etag", file.filename()));
        fs::read_to_string(etag_path).ok()
    }

    /// Save the ETag for a downloaded file.
    fn save_etag(&self, file_path: &Path, etag: &str) -> Result<()> {
        let etag_path = file_path.with_extension("zip.etag");
        let mut file = File::create(etag_path)?;
        file.write_all(etag.as_bytes())?;
        Ok(())
    }

    /// Get the modification time of a cached file.
    ///
    /// Returns the file's modification time (which should match the FCC server's
    /// Last-Modified header if we preserved it), or None if the file doesn't exist.
    pub fn get_cached_file_date(&self, file: &DataFile) -> Option<chrono::DateTime<Utc>> {
        let path = self.cache_path(file);
        std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(chrono::DateTime::<Utc>::from)
    }
}

/// Result of a download operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadResult {
    /// File was newly downloaded.
    Downloaded,
    /// File was not modified (HTTP 304) - cache is valid.
    NotModified,
}

/// Information about available updates.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The data file.
    pub file: DataFile,
    /// Size of the file in bytes.
    pub size_bytes: Option<u64>,
    /// Last-Modified header value.
    pub last_modified: Option<String>,
    /// ETag header value.
    pub etag: Option<String>,
    /// Whether an update is available.
    pub has_update: bool,
    /// When this check was performed.
    pub checked_at: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Weekday;
    use tempfile::TempDir;

    #[test]
    fn test_url_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf());
        let client = FccClient::new(config).unwrap();

        let file = DataFile::complete_license("amat");
        assert!(client.url(&file).ends_with("/complete/l_amat.zip"));

        let daily = DataFile::daily_license("amat", Weekday::Monday);
        assert!(client.url(&daily).ends_with("/daily/l_am_mon.zip"));
    }

    #[test]
    fn test_cache_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig::with_cache_dir(temp_dir.path().to_path_buf());
        let client = FccClient::new(config).unwrap();

        let file = DataFile::complete_license("amat");
        assert_eq!(client.cache_path(&file), temp_dir.path().join("l_amat.zip"));
    }
}
