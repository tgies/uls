//! Database adapters for enum columns.
//!
//! These helpers read integer-encoded enum columns from database rows
//! and convert them back to their string/char representations.

use rusqlite::Row;
use uls_core::codes::{LicenseStatus, OperatorClass, RadioService};

/// Read a license_status column (stored as INTEGER) and return as char.
///
/// Returns '?' if the value is NULL or unrecognized.
pub fn read_license_status(row: &Row, idx: usize) -> rusqlite::Result<char> {
    let code: Option<i64> = row.get(idx)?;
    Ok(code
        .and_then(|c| LicenseStatus::from_u8(c as u8))
        .map(|s| s.as_str().chars().next().unwrap_or('?'))
        .unwrap_or('?'))
}

/// Read a radio_service_code column (stored as INTEGER) and return as String.
///
/// Returns empty string if the value is NULL or unrecognized.
pub fn read_radio_service(row: &Row, idx: usize) -> rusqlite::Result<String> {
    let code: Option<i64> = row.get(idx)?;
    Ok(code
        .and_then(|c| RadioService::from_u8(c as u8))
        .map(|r| r.as_str().to_string())
        .unwrap_or_default())
}

/// Read an operator_class column (stored as INTEGER) and return as `Option<char>.
///
/// Returns None if the value is NULL or unrecognized.
pub fn read_operator_class(row: &Row, idx: usize) -> rusqlite::Result<Option<char>> {
    let code: Option<i64> = row.get(idx)?;
    Ok(code
        .and_then(|c| OperatorClass::from_u8(c as u8))
        .map(|o| o.as_str().chars().next().unwrap_or('?')))
}

#[cfg(test)]
mod tests {
    // Note: These would require mocking rusqlite::Row which is complex.
    // The actual behavior is tested via integration tests.
}
