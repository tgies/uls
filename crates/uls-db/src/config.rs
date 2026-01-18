//! Database configuration.

use std::path::PathBuf;
use std::time::Duration;

/// Database configuration options.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file.
    pub path: PathBuf,

    /// Maximum number of connections in the pool.
    pub max_connections: u32,

    /// Connection timeout.
    pub connection_timeout: Duration,

    /// Whether to create the database if it doesn't exist.
    pub create_if_missing: bool,

    /// Enable WAL mode for better concurrent read performance.
    pub enable_wal: bool,

    /// Cache size in pages (negative = KB).
    pub cache_size: i32,

    /// Enable foreign key constraints.
    pub foreign_keys: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
            max_connections: 4,
            connection_timeout: Duration::from_secs(30),
            create_if_missing: true,
            enable_wal: true,
            cache_size: -64000, // 64MB
            foreign_keys: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a config with a specific database path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Create an in-memory database configuration (for testing).
    pub fn in_memory() -> Self {
        Self {
            path: PathBuf::from(":memory:"),
            max_connections: 1, // In-memory only works with single connection
            create_if_missing: true,
            enable_wal: false, // WAL not supported for in-memory
            ..Default::default()
        }
    }

    /// Set maximum connections.
    pub fn with_max_connections(mut self, count: u32) -> Self {
        self.max_connections = count;
        self
    }

    /// Set cache size in megabytes.
    pub fn with_cache_size_mb(mut self, mb: i32) -> Self {
        self.cache_size = -mb * 1000;
        self
    }
}

/// Get the default database path.
fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.enable_wal);
        assert!(config.create_if_missing);
        assert_eq!(config.max_connections, 4);
    }

    #[test]
    fn test_in_memory_config() {
        let config = DatabaseConfig::in_memory();
        assert_eq!(config.path.to_str(), Some(":memory:"));
        assert_eq!(config.max_connections, 1);
        assert!(!config.enable_wal);
    }

    #[test]
    fn test_builder_pattern() {
        let config = DatabaseConfig::default()
            .with_max_connections(8)
            .with_cache_size_mb(128);

        assert_eq!(config.max_connections, 8);
        // Cache size is stored as negative KB, so 128MB = -128000
        assert_eq!(config.cache_size, -128000);
    }

    #[test]
    fn test_with_max_connections() {
        let config = DatabaseConfig::in_memory().with_max_connections(16);
        assert_eq!(config.max_connections, 16);
    }

    #[test]
    fn test_with_cache_size_mb() {
        let config = DatabaseConfig::in_memory().with_cache_size_mb(256);
        // 256MB * 1000 = 256000 pages, stored as negative
        assert_eq!(config.cache_size, -256000);
    }
}
