//! Error types for ULS core operations.

use thiserror::Error;

/// Core ULS error type.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid record type encountered.
    #[error("invalid record type: {0}")]
    InvalidRecordType(String),

    /// Failed to parse a field value.
    #[error("failed to parse field '{field}': {message}")]
    ParseField {
        field: &'static str,
        message: String,
    },

    /// Invalid date format.
    #[error("invalid date format: {0}")]
    InvalidDate(String),

    /// Invalid enum value.
    #[error("invalid {enum_type} value: {value}")]
    InvalidEnumValue {
        enum_type: &'static str,
        value: String,
    },

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    /// Field count mismatch.
    #[error("expected {expected} fields, got {actual} for record type {record_type}")]
    FieldCountMismatch {
        record_type: String,
        expected: usize,
        actual: usize,
    },
}

/// Result type alias for ULS core operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidRecordType("XX".to_string());
        assert_eq!(err.to_string(), "invalid record type: XX");

        let err = Error::ParseField {
            field: "frequency",
            message: "not a number".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "failed to parse field 'frequency': not a number"
        );

        let err = Error::InvalidDate("2024-13-45".to_string());
        assert_eq!(err.to_string(), "invalid date format: 2024-13-45");

        let err = Error::InvalidEnumValue {
            enum_type: "RadioService",
            value: "ZZ".to_string(),
        };
        assert_eq!(err.to_string(), "invalid RadioService value: ZZ");

        let err = Error::MissingField("call_sign");
        assert_eq!(err.to_string(), "missing required field: call_sign");

        let err = Error::FieldCountMismatch {
            record_type: "HD".to_string(),
            expected: 60,
            actual: 55,
        };
        assert_eq!(
            err.to_string(),
            "expected 60 fields, got 55 for record type HD"
        );
    }
}
