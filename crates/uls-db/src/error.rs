//! Database error types.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during database operations.
#[derive(Error, Debug)]
pub enum DbError {
    /// SQLite error.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Connection pool error.
    #[error("connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Record not found.
    #[error("record not found: {0}")]
    NotFound(String),

    /// Database not initialized.
    #[error("database not initialized - run 'uls update' first")]
    NotInitialized,

    /// Schema version mismatch.
    #[error("schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: i32, found: i32 },

    /// Database file does not exist.
    #[error("database file not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid data.
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// Transaction error.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Parser error.
    #[error("parser error: {0}")]
    Parser(#[from] uls_parser::ParseError),

    /// ZIP archive error.
    #[error("zip error: {0}")]
    Zip(#[from] uls_parser::archive::ZipError),
}

/// Result type for database operations.
pub type Result<T> = std::result::Result<T, DbError>;
