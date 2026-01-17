//! SC/SF (Special Conditions) record types.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialConditionRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub callsign: Option<String>,
    pub special_condition_type: Option<char>,
    pub special_condition_code: Option<i32>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
}

impl SpecialConditionRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            special_condition_type: parse_opt_char(fields.get(5).unwrap_or(&"")),
            special_condition_code: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(7).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(8).unwrap_or(&"")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeformConditionRecord {
    pub unique_system_identifier: Option<i64>,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub callsign: Option<String>,
    pub lic_freeform_cond_type: Option<char>,
    pub unique_lic_freeform_id: Option<i64>,
    pub sequence_number: Option<i32>,
    pub lic_freeform_condition: Option<String>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
}

impl FreeformConditionRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_opt_i64(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            lic_freeform_cond_type: parse_opt_char(fields.get(5).unwrap_or(&"")),
            unique_lic_freeform_id: parse_opt_i64(fields.get(6).unwrap_or(&"")),
            sequence_number: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            lic_freeform_condition: parse_opt_string(fields.get(8).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(9).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(10).unwrap_or(&"")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityCallSignRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub request_sequence: Option<i32>,
    pub callsign_requested: Option<String>,
}

impl VanityCallSignRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            request_sequence: parse_opt_i32(fields.get(4).unwrap_or(&"")),
            callsign_requested: parse_opt_string(fields.get(5).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_condition_from_fields() {
        let fields = vec!["SC", "12345", "ULS123", "EBF456", "W1TEST", "P", "999", "A", "01/01/2020"];
        let sc = SpecialConditionRecord::from_fields(&fields);
        
        assert_eq!(sc.unique_system_identifier, 12345);
        assert_eq!(sc.uls_file_number, Some("ULS123".to_string()));
        assert_eq!(sc.callsign, Some("W1TEST".to_string()));
        assert_eq!(sc.special_condition_type, Some('P'));
        assert_eq!(sc.special_condition_code, Some(999));
        assert_eq!(sc.status_code, Some('A'));
    }

    #[test]
    fn test_special_condition_from_minimal_fields() {
        let fields = vec!["SC", "12345"];
        let sc = SpecialConditionRecord::from_fields(&fields);
        
        assert_eq!(sc.unique_system_identifier, 12345);
        assert!(sc.callsign.is_none());
        assert!(sc.special_condition_code.is_none());
    }

    #[test]
    fn test_freeform_condition_from_fields() {
        let fields = vec![
            "SF", "12345", "ULS123", "EBF456", "W1TEST", "P",
            "999", "1", "This is a test condition", "A", "01/01/2020"
        ];
        let sf = FreeformConditionRecord::from_fields(&fields);
        
        assert_eq!(sf.unique_system_identifier, Some(12345));
        assert_eq!(sf.callsign, Some("W1TEST".to_string()));
        assert_eq!(sf.lic_freeform_cond_type, Some('P'));
        assert_eq!(sf.unique_lic_freeform_id, Some(999));
        assert_eq!(sf.sequence_number, Some(1));
        assert_eq!(sf.lic_freeform_condition, Some("This is a test condition".to_string()));
    }

    #[test]
    fn test_vanity_callsign_from_fields() {
        let fields = vec!["VC", "12345", "ULS123", "EBF456", "1", "W1AW"];
        let vc = VanityCallSignRecord::from_fields(&fields);
        
        assert_eq!(vc.unique_system_identifier, 12345);
        assert_eq!(vc.request_sequence, Some(1));
        assert_eq!(vc.callsign_requested, Some("W1AW".to_string()));
    }
}

