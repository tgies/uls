//! AM (Amateur) record type - Amateur operator data.

use serde::{Deserialize, Serialize};

use super::common::*;

/// AM record - Amateur operator data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmateurRecord {
    pub unique_system_identifier: i64,
    pub uls_file_num: Option<String>,
    pub ebf_number: Option<String>,
    pub callsign: Option<String>,
    pub operator_class: Option<char>,
    pub group_code: Option<char>,
    pub region_code: Option<i32>,
    pub trustee_callsign: Option<String>,
    pub trustee_indicator: Option<char>,
    pub physician_certification: Option<char>,
    pub ve_signature: Option<char>,
    pub systematic_callsign_change: Option<char>,
    pub vanity_callsign_change: Option<char>,
    pub vanity_relationship: Option<String>,
    pub previous_callsign: Option<String>,
    pub previous_operator_class: Option<char>,
    pub trustee_name: Option<String>,
}

impl AmateurRecord {
    /// Parse an amateur record from pipe-delimited fields.
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_num: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            operator_class: parse_opt_char(fields.get(5).unwrap_or(&"")),
            group_code: parse_opt_char(fields.get(6).unwrap_or(&"")),
            region_code: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            trustee_callsign: parse_opt_string(fields.get(8).unwrap_or(&"")),
            trustee_indicator: parse_opt_char(fields.get(9).unwrap_or(&"")),
            physician_certification: parse_opt_char(fields.get(10).unwrap_or(&"")),
            ve_signature: parse_opt_char(fields.get(11).unwrap_or(&"")),
            systematic_callsign_change: parse_opt_char(fields.get(12).unwrap_or(&"")),
            vanity_callsign_change: parse_opt_char(fields.get(13).unwrap_or(&"")),
            vanity_relationship: parse_opt_string(fields.get(14).unwrap_or(&"")),
            previous_callsign: parse_opt_string(fields.get(15).unwrap_or(&"")),
            previous_operator_class: parse_opt_char(fields.get(16).unwrap_or(&"")),
            trustee_name: parse_opt_string(fields.get(17).unwrap_or(&"")),
        }
    }

    /// Returns true if this is a club station (has trustee).
    pub fn is_club(&self) -> bool {
        self.trustee_indicator == Some('Y') || self.trustee_callsign.is_some()
    }

    /// Returns the operator class as a descriptive string.
    pub fn operator_class_description(&self) -> Option<&'static str> {
        match self.operator_class {
            Some('A') => Some("Advanced"),
            Some('E') => Some("Amateur Extra"),
            Some('G') => Some("General"),
            Some('N') => Some("Novice"),
            Some('P') => Some("Technician Plus"),
            Some('T') => Some("Technician"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amateur_from_fields() {
        let fields: Vec<&str> = vec![
            "AM",
            "123456789",
            "",
            "",
            "W1AW",
            "E",
            "D",
            "1",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ];
        let record = AmateurRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 123456789);
        assert_eq!(record.callsign, Some("W1AW".to_string()));
        assert_eq!(record.operator_class, Some('E'));
        assert_eq!(record.operator_class_description(), Some("Amateur Extra"));
        assert!(!record.is_club());
    }

    #[test]
    fn test_club_station() {
        let mut record = AmateurRecord::from_fields(&["AM", "123"]);
        record.trustee_indicator = Some('Y');
        record.trustee_callsign = Some("N1MM".to_string());
        assert!(record.is_club());
    }
}
