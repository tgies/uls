//! Error types for download operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during download operations.
#[derive(Error, Debug)]
pub enum DownloadError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP extraction error.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Unknown service code.
    #[error("unknown service code: {0}")]
    UnknownService(String),

    /// File not found on FCC server.
    #[error("file not found: {url}")]
    NotFound { url: String },

    /// Server returned an error status.
    #[error("server error {status} for {url}")]
    ServerError { status: u16, url: String },

    /// Download was interrupted or incomplete.
    #[error("incomplete download: expected {expected} bytes, got {actual}")]
    IncompleteDownload { expected: u64, actual: u64 },

    /// Cache directory could not be created.
    #[error("failed to create cache directory: {path}")]
    CacheDirectoryError { path: PathBuf },

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for download operations.
pub type Result<T> = std::result::Result<T, DownloadError>;
