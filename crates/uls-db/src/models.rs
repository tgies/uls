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

    /// Get a field value by name for dynamic output.
    pub fn get_field(&self, name: &str) -> Option<String> {
        match name.to_lowercase().as_str() {
            "call_sign" | "callsign" | "call" => Some(self.call_sign.clone()),
            "name" | "licensee" | "entity_name" => Some(self.display_name()),
            "first_name" | "first" => self.first_name.clone(),
            "last_name" | "last" => self.last_name.clone(),
            "middle_initial" | "mi" => self.middle_initial.clone(),
            "status" | "license_status" => Some(self.status.to_string()),
            "status_desc" | "status_description" => Some(self.status_description().to_string()),
            "service" | "radio_service" => Some(self.radio_service.clone()),
            "class" | "operator_class" => self.operator_class.map(|c| c.to_string()),
            "class_desc" | "class_description" => {
                self.operator_class_description().map(|s| s.to_string())
            }
            "city" => self.city.clone(),
            "state" => self.state.clone(),
            "zip" | "zip_code" => self.zip_code.clone(),
            "location" => {
                let city = self.city.as_deref().unwrap_or("");
                let state = self.state.as_deref().unwrap_or("");
                if city.is_empty() && state.is_empty() {
                    None
                } else {
                    Some(format!("{}, {}", city, state))
                }
            }
            "address" | "street_address" => self.street_address.clone(),
            "frn" => self.frn.clone(),
            "grant_date" | "granted" => self.grant_date.map(|d| d.to_string()),
            "expired_date" | "expires" | "expiration" => self.expired_date.map(|d| d.to_string()),
            "cancellation_date" | "cancelled" => self.cancellation_date.map(|d| d.to_string()),
            "previous_call_sign" | "previous_call" => self.previous_call_sign.clone(),
            "usi" | "unique_system_identifier" => Some(self.unique_system_identifier.to_string()),
            _ => None,
        }
    }

    /// Get list of available field names.
    pub fn field_names() -> &'static [&'static str] {
        &[
            "call_sign",
            "name",
            "first_name",
            "last_name",
            "status",
            "status_desc",
            "service",
            "class",
            "class_desc",
            "city",
            "state",
            "zip",
            "location",
            "address",
            "frn",
            "grant_date",
            "expired_date",
            "cancellation_date",
            "previous_call_sign",
            "usi",
        ]
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

    #[test]
    fn test_status_description_all_variants() {
        let mut license = License {
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
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(license.status_description(), "Active");

        license.status = 'C';
        assert_eq!(license.status_description(), "Cancelled");

        license.status = 'E';
        assert_eq!(license.status_description(), "Expired");

        license.status = 'L';
        assert_eq!(license.status_description(), "Pending Legal Status");

        license.status = 'P';
        assert_eq!(license.status_description(), "Parent Station Cancelled");

        license.status = 'T';
        assert_eq!(license.status_description(), "Terminated");

        license.status = 'X';
        assert_eq!(license.status_description(), "Term Pending");

        license.status = 'Z';
        assert_eq!(license.status_description(), "Unknown");
    }

    #[test]
    fn test_operator_class_all_variants() {
        let mut license = License {
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
            operator_class: Some('T'),
            previous_call_sign: None,
        };

        assert_eq!(license.operator_class_description(), Some("Technician"));

        license.operator_class = Some('G');
        assert_eq!(license.operator_class_description(), Some("General"));

        license.operator_class = Some('A');
        assert_eq!(license.operator_class_description(), Some("Advanced"));

        license.operator_class = Some('N');
        assert_eq!(license.operator_class_description(), Some("Novice"));

        license.operator_class = Some('P');
        assert_eq!(
            license.operator_class_description(),
            Some("Technician Plus")
        );

        license.operator_class = Some('Z');
        assert_eq!(license.operator_class_description(), Some("Unknown"));

        license.operator_class = None;
        assert_eq!(license.operator_class_description(), None);
    }

    #[test]
    fn test_display_name_without_middle_initial() {
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "ARRL".to_string(),
            first_name: Some("John".to_string()),
            middle_initial: None,
            last_name: Some("Doe".to_string()),
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
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(license.display_name(), "John Doe");
    }

    #[test]
    fn test_get_field_basic() {
        let license = License {
            unique_system_identifier: 12345,
            call_sign: "W1AW".to_string(),
            licensee_name: "ARRL".to_string(),
            first_name: Some("John".to_string()),
            middle_initial: Some("Q".to_string()),
            last_name: Some("Public".to_string()),
            status: 'A',
            radio_service: "HA".to_string(),
            grant_date: Some(NaiveDate::from_ymd_opt(2020, 1, 15).unwrap()),
            expired_date: Some(NaiveDate::from_ymd_opt(2030, 1, 15).unwrap()),
            cancellation_date: None,
            frn: Some("0012345678".to_string()),
            street_address: Some("123 Main St".to_string()),
            city: Some("Newington".to_string()),
            state: Some("CT".to_string()),
            zip_code: Some("06111".to_string()),
            operator_class: Some('E'),
            previous_call_sign: Some("N1XYZ".to_string()),
        };

        // Test all field name variants
        assert_eq!(license.get_field("call_sign"), Some("W1AW".to_string()));
        assert_eq!(license.get_field("callsign"), Some("W1AW".to_string()));
        assert_eq!(license.get_field("call"), Some("W1AW".to_string()));

        assert_eq!(license.get_field("name"), Some("John Q Public".to_string()));
        assert_eq!(
            license.get_field("licensee"),
            Some("John Q Public".to_string())
        );

        assert_eq!(license.get_field("first_name"), Some("John".to_string()));
        assert_eq!(license.get_field("first"), Some("John".to_string()));

        assert_eq!(license.get_field("last_name"), Some("Public".to_string()));
        assert_eq!(license.get_field("last"), Some("Public".to_string()));

        assert_eq!(license.get_field("middle_initial"), Some("Q".to_string()));
        assert_eq!(license.get_field("mi"), Some("Q".to_string()));

        assert_eq!(license.get_field("status"), Some("A".to_string()));
        assert_eq!(license.get_field("status_desc"), Some("Active".to_string()));

        assert_eq!(license.get_field("service"), Some("HA".to_string()));
        assert_eq!(license.get_field("radio_service"), Some("HA".to_string()));

        assert_eq!(license.get_field("class"), Some("E".to_string()));
        assert_eq!(
            license.get_field("class_desc"),
            Some("Amateur Extra".to_string())
        );

        assert_eq!(license.get_field("city"), Some("Newington".to_string()));
        assert_eq!(license.get_field("state"), Some("CT".to_string()));
        assert_eq!(license.get_field("zip"), Some("06111".to_string()));
        assert_eq!(license.get_field("zip_code"), Some("06111".to_string()));

        assert_eq!(
            license.get_field("location"),
            Some("Newington, CT".to_string())
        );

        assert_eq!(
            license.get_field("address"),
            Some("123 Main St".to_string())
        );
        assert_eq!(
            license.get_field("street_address"),
            Some("123 Main St".to_string())
        );

        assert_eq!(license.get_field("frn"), Some("0012345678".to_string()));

        assert_eq!(
            license.get_field("grant_date"),
            Some("2020-01-15".to_string())
        );
        assert_eq!(license.get_field("granted"), Some("2020-01-15".to_string()));

        assert_eq!(
            license.get_field("expired_date"),
            Some("2030-01-15".to_string())
        );
        assert_eq!(license.get_field("expires"), Some("2030-01-15".to_string()));

        assert_eq!(license.get_field("cancellation_date"), None);
        assert_eq!(license.get_field("cancelled"), None);

        assert_eq!(
            license.get_field("previous_call_sign"),
            Some("N1XYZ".to_string())
        );
        assert_eq!(
            license.get_field("previous_call"),
            Some("N1XYZ".to_string())
        );

        assert_eq!(license.get_field("usi"), Some("12345".to_string()));
        assert_eq!(
            license.get_field("unique_system_identifier"),
            Some("12345".to_string())
        );

        // Unknown field
        assert_eq!(license.get_field("unknown_field"), None);
    }

    #[test]
    fn test_get_field_location_empty() {
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
            operator_class: None,
            previous_call_sign: None,
        };

        // Empty city and state should return None for location
        assert_eq!(license.get_field("location"), None);
    }

    #[test]
    fn test_get_field_no_operator_class() {
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "Test".to_string(),
            first_name: None,
            middle_initial: None,
            last_name: None,
            status: 'A',
            radio_service: "ZA".to_string(), // GMRS has no class
            grant_date: None,
            expired_date: None,
            cancellation_date: None,
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(license.get_field("class"), None);
        assert_eq!(license.get_field("class_desc"), None);
    }

    #[test]
    fn test_field_names() {
        let names = License::field_names();
        assert!(names.contains(&"call_sign"));
        assert!(names.contains(&"name"));
        assert!(names.contains(&"status"));
        assert!(names.contains(&"city"));
        assert!(names.contains(&"state"));
        assert!(names.contains(&"frn"));
        assert!(names.contains(&"grant_date"));
        assert!(names.len() >= 15);
    }

    #[test]
    fn test_is_active() {
        let mut license = License {
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
            operator_class: None,
            previous_call_sign: None,
        };

        assert!(license.is_active());

        license.status = 'E';
        assert!(!license.is_active());

        license.status = 'C';
        assert!(!license.is_active());
    }

    #[test]
    fn test_get_field_cancellation_date_with_value() {
        // Tests the cancellation_date path when a date IS present
        // (the lambda inside .map() was never exercised before)
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "Test".to_string(),
            first_name: None,
            middle_initial: None,
            last_name: None,
            status: 'C',
            radio_service: "HA".to_string(),
            grant_date: None,
            expired_date: None,
            cancellation_date: Some(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap()),
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(
            license.get_field("cancellation_date"),
            Some("2023-06-15".to_string())
        );
        assert_eq!(
            license.get_field("cancelled"),
            Some("2023-06-15".to_string())
        );
    }

    #[test]
    fn test_get_field_location_partial() {
        // Tests location field with only city or only state present
        let mut license = License {
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
            city: Some("Boston".to_string()),
            state: None,
            zip_code: None,
            operator_class: None,
            previous_call_sign: None,
        };

        // Only city, no state - should still format
        assert_eq!(license.get_field("location"), Some("Boston, ".to_string()));

        // Only state, no city
        license.city = None;
        license.state = Some("MA".to_string());
        assert_eq!(license.get_field("location"), Some(", MA".to_string()));
    }

    #[test]
    fn test_get_field_expiration_alias() {
        // Ensure the "expiration" alias works for expired_date
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
            expired_date: Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()),
            cancellation_date: None,
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(
            license.get_field("expiration"),
            Some("2030-12-31".to_string())
        );
    }

    #[test]
    fn test_get_field_entity_name_alias() {
        // Ensure the "entity_name" alias works for display_name
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "ARRL HQ Station".to_string(),
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
            operator_class: None,
            previous_call_sign: None,
        };

        // When no first/last name, should return licensee_name
        assert_eq!(
            license.get_field("entity_name"),
            Some("ARRL HQ Station".to_string())
        );
    }

    #[test]
    fn test_get_field_license_status_alias() {
        // Ensure the "license_status" alias works for status
        let license = License {
            unique_system_identifier: 1,
            call_sign: "W1AW".to_string(),
            licensee_name: "Test".to_string(),
            first_name: None,
            middle_initial: None,
            last_name: None,
            status: 'E',
            radio_service: "HA".to_string(),
            grant_date: None,
            expired_date: None,
            cancellation_date: None,
            frn: None,
            street_address: None,
            city: None,
            state: None,
            zip_code: None,
            operator_class: None,
            previous_call_sign: None,
        };

        assert_eq!(license.get_field("license_status"), Some("E".to_string()));
    }
}
