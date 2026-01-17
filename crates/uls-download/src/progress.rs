//! Progress tracking and callbacks for downloads.

use std::sync::Arc;

/// Information about download progress.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download (if known).
    pub total_bytes: Option<u64>,

    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,

    /// Current download speed in bytes per second.
    pub speed_bps: u64,

    /// Estimated time remaining in seconds.
    pub eta_seconds: Option<u64>,

    /// Name of the file being downloaded.
    pub filename: String,
}

impl DownloadProgress {
    /// Create a new progress tracker.
    pub fn new(filename: impl Into<String>, total_bytes: Option<u64>) -> Self {
        Self {
            total_bytes,
            downloaded_bytes: 0,
            speed_bps: 0,
            eta_seconds: None,
            filename: filename.into(),
        }
    }

    /// Get the completion percentage (0.0 to 1.0).
    pub fn fraction(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 {
                1.0
            } else {
                self.downloaded_bytes as f64 / total as f64
            }
        })
    }

    /// Get the completion percentage as an integer (0 to 100).
    pub fn percent(&self) -> Option<u8> {
        self.fraction().map(|f| (f * 100.0).min(100.0) as u8)
    }

    /// Format the download speed as a human-readable string.
    pub fn speed_string(&self) -> String {
        format_bytes_per_second(self.speed_bps)
    }

    /// Format the downloaded/total as a human-readable string.
    pub fn size_string(&self) -> String {
        match self.total_bytes {
            Some(total) => format!(
                "{} / {}",
                format_bytes(self.downloaded_bytes),
                format_bytes(total)
            ),
            None => format_bytes(self.downloaded_bytes),
        }
    }
}

/// Callback function type for progress updates.
pub type ProgressCallback = Arc<dyn Fn(&DownloadProgress) + Send + Sync>;

/// Create a no-op progress callback.
pub fn no_progress() -> ProgressCallback {
    Arc::new(|_| {})
}

/// Format bytes as a human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format bytes per second as a human-readable string.
fn format_bytes_per_second(bps: u64) -> String {
    format!("{}/s", format_bytes(bps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_fraction() {
        let mut progress = DownloadProgress::new("test.zip", Some(1000));
        progress.downloaded_bytes = 500;

        assert_eq!(progress.fraction(), Some(0.5));
        assert_eq!(progress.percent(), Some(50));
    }

    #[test]
    fn test_progress_unknown_total() {
        let progress = DownloadProgress::new("test.zip", None);
        assert_eq!(progress.fraction(), None);
        assert_eq!(progress.percent(), None);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_size_string() {
        let mut progress = DownloadProgress::new("test.zip", Some(1024 * 1024));
        progress.downloaded_bytes = 512 * 1024;

        assert_eq!(progress.size_string(), "512.0 KB / 1.00 MB");
    }
}
