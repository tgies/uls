//! Parser for FCC ULS pipe-delimited DAT files.
//!
//! This crate provides functionality to parse the DAT files extracted from
//! FCC ULS ZIP archives. Each DAT file contains pipe-delimited records
//! with the record type as the first field.

pub mod archive;
pub mod dat;

pub use archive::ZipExtractor;
pub use dat::{DatReader, ParsedLine};

use thiserror::Error;

/// Parser error types.
#[derive(Error, Debug)]
pub enum ParseError {
    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid record format.
    #[error("invalid record format on line {line}: {message}")]
    InvalidFormat { line: usize, message: String },

    /// Unknown record type.
    #[error("unknown record type: {0}")]
    UnknownRecordType(String),

    /// Field parsing error.
    #[error("failed to parse field {field} on line {line}: {message}")]
    FieldParse {
        line: usize,
        field: String,
        message: String,
    },

    /// ZIP extraction error.
    #[error("ZIP error: {0}")]
    Zip(#[from] ::zip::result::ZipError),
}

/// Result type for parser operations.
pub type Result<T> = std::result::Result<T, ParseError>;
