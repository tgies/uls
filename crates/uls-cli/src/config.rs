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

#[cfg(test)]
mod tests {
    use super::*;

    /// `ULS_DB_PATH` is process-global, so the fallback and env-override
    /// branches run in one test to avoid cross-test interference.
    #[test]
    fn test_default_db_path_env_branches() {
        // Save and clear so we observe the fallback first.
        let saved = std::env::var("ULS_DB_PATH").ok();
        std::env::remove_var("ULS_DB_PATH");

        let fallback = default_db_path();
        assert!(
            fallback.ends_with("uls/uls.db") || fallback.ends_with("uls\\uls.db"),
            "fallback should end with uls/uls.db, got {}",
            fallback.display()
        );

        // Explicit env var takes precedence verbatim.
        std::env::set_var("ULS_DB_PATH", "/tmp/custom/explicit.db");
        assert_eq!(default_db_path(), PathBuf::from("/tmp/custom/explicit.db"));

        // Restore prior state.
        match saved {
            Some(v) => std::env::set_var("ULS_DB_PATH", v),
            None => std::env::remove_var("ULS_DB_PATH"),
        }
    }

    #[test]
    fn test_default_cache_path_ends_with_uls() {
        let path = default_cache_path();
        let last = path.file_name().and_then(|s| s.to_str());
        assert_eq!(last, Some("uls"));
    }
}
