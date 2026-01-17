//! High-level domain models for ULS data.
//!
//! These models aggregate data from multiple record types into
//! user-friendly structures for queries and display.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A complete license with all related information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// Unique system identifier.
    pub unique_system_identifier: i64,
    /// Call sign.
    pub call_sign: String,
    /// Licensee/entity name.
    pub licensee_name: String,
    /// First name (if individual).
    pub first_name: Option<String>,
    /// Middle initial.
    pub middle_initial: Option<String>,
    /// Last name (if individual).
    pub last_name: Option<String>,
    /// License status (A=Active, C=Cancelled, E=Expired, etc.).
    pub status: char,
    /// Radio service code (HA, HV, ZA, etc.).
    pub radio_service: String,
    /// Grant date.
    pub grant_date: Option<NaiveDate>,
    /// Expiration date.
    pub expired_date: Option<NaiveDate>,
    /// Cancellation date.
    pub cancellation_date: Option<NaiveDate>,
    /// FRN (FCC Registration Number).
    pub frn: Option<String>,
    /// Street address.
    pub street_address: Option<String>,
    /// City.
    pub city: Option<String>,
    /// State.
    pub state: Option<String>,
    /// ZIP code.
    pub zip_code: Option<String>,
    /// Operator class (for amateur).
    pub operator_class: Option<char>,
    /// Previous call sign.
    pub previous_call_sign: Option<String>,
}

impl License {
    /// Get the formatted name (entity name or "First Last").
    pub fn display_name(&self) -> String {
        if let (Some(first), Some(last)) = (&self.first_name, &self.last_name) {
            if let Some(mi) = &self.middle_initial {
                format!("{} {} {}", first, mi, last)
            } else {
                format!("{} {}", first, last)
            }
        } else {
            self.licensee_name.clone()
        }
    }

    /// Get the status description.
    pub fn status_description(&self) -> &'static str {
        match self.status {
            'A' => "Active",
            'C' => "Cancelled",
            'E' => "Expired",
            'L' => "Pending Legal Status",
            'P' => "Parent Station Cancelled",
            'T' => "Terminated",
            'X' => "Term Pending",
            _ => "Unknown",
        }
    }

    /// Check if the license is active.
    pub fn is_active(&self) -> bool {
        self.status == 'A'
    }

    /// Get the operator class description (amateur only).
    pub fn operator_class_description(&self) -> Option<&'static str> {
        self.operator_class.map(|c| match c {
            'T' => "Technician",
            'G' => "General",
            'A' => "Advanced",
            'E' => "Amateur Extra",
            'N' => "Novice",
            'P' => "Technician Plus",
            _ => "Unknown",
        })
    }
}

/// Amateur operator information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    /// Unique system identifier.
    pub unique_system_identifier: i64,
    /// Call sign.
    pub call_sign: String,
    /// Operator class.
    pub operator_class: char,
    /// Group code.
    pub group_code: Option<char>,
    /// Region code.
    pub region_code: Option<i32>,
    /// Vanity call indicator.
    pub vanity_call: bool,
    /// Previous operator class.
    pub previous_operator_class: Option<char>,
}

/// Database statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStats {
    /// Total number of licenses.
    pub total_licenses: u64,
    /// Number of active licenses.
    pub active_licenses: u64,
    /// Number of expired licenses.
    pub expired_licenses: u64,
    /// Number of cancelled licenses.
    pub cancelled_licenses: u64,
    /// Breakdown by radio service.
    pub by_service: Vec<(String, u64)>,
    /// Breakdown by operator class (amateur only).
    pub by_operator_class: Vec<(String, u64)>,
    /// Database last updated.
    pub last_updated: Option<String>,
    /// Database schema version.
    pub schema_version: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_display_name() {
        let mut license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "ARRL".to_string(),
            first_name: Some("Test".to_string()),
            middle_initial: Some("X".to_string()),
            last_name: Some("User".to_string()),
            status: 'A',
            radio_service: "HA".to_string(),
            grant_date: None,
            expired_date: None,
            cancellation_date: None,
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: Some('E'),
            previous_call_sign: None,
        };

        assert_eq!(license.display_name(), "Test X User");

        license.first_name = None;
        assert_eq!(license.display_name(), "ARRL");
    }

    #[test]
    fn test_operator_class_description() {
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "Test".to_string(),
            first_name: None,
            middle_initial: None,
            last_name: None,
            status: 'A',
            radio_service: "HA".to_string(),
            grant_date: None,
            expired_date: None,
            cancellation_date: None,
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: Some('E'),
            previous_call_sign: None,
        };

        assert_eq!(license.operator_class_description(), Some("Amateur Extra"));
        assert!(license.is_active());
    }
}
