//! Shared CLI configuration.
//!
//! Configuration values and paths used across CLI commands.

use std::path::PathBuf;

/// Get the default database path.
///
/// Checks `ULS_DB_PATH` env var first, then falls back to system data dir.
pub fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("ULS_DB_PATH") {
        return PathBuf::from(path);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

/// Get the default cache path for downloads.
pub fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("uls")
}
