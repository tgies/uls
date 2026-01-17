//! Download configuration.

use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the FCC download client.
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Directory to cache downloaded files.
    pub cache_dir: PathBuf,

    /// Base URL for FCC ULS data.
    pub base_url: String,

    /// HTTP request timeout.
    pub timeout: Duration,

    /// User-Agent header to send with requests.
    pub user_agent: String,

    /// Whether to verify SSL certificates.
    pub verify_ssl: bool,

    /// Maximum number of retries for failed requests.
    pub max_retries: u32,

    /// Delay between retries.
    pub retry_delay: Duration,

    /// Maximum download bandwidth in bytes per second (0 = unlimited).
    pub bandwidth_limit: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
            base_url: "https://data.fcc.gov/download/pub/uls".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes
            user_agent: format!("uls-cli/{} (https://github.com/tgies/uls)", env!("CARGO_PKG_VERSION")),
            verify_ssl: true,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            bandwidth_limit: 0,
        }
    }
}

impl DownloadConfig {
    /// Create a new configuration with custom cache directory.
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            ..Default::default()
        }
    }

    /// Set the base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the User-Agent header.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set bandwidth limit in bytes per second.
    pub fn with_bandwidth_limit(mut self, bytes_per_second: u64) -> Self {
        self.bandwidth_limit = bytes_per_second;
        self
    }
}

/// Get the default cache directory.
fn default_cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uls")
        .join("cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DownloadConfig::default();
        assert!(config.base_url.contains("data.fcc.gov"));
        assert!(config.user_agent.contains("uls-cli"));
        assert!(config.verify_ssl);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_builder_pattern() {
        let config = DownloadConfig::default()
            .with_timeout(Duration::from_secs(60))
            .with_bandwidth_limit(1_000_000);

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.bandwidth_limit, 1_000_000);
    }
}
