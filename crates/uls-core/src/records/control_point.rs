//! CP (Control Point) record type.
use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPointRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub control_point_action_performed: Option<char>,
    pub control_point_number: Option<i32>,
    pub control_address: Option<String>,
    pub control_city: Option<String>,
    pub state_code: Option<String>,
    pub control_phone: Option<String>,
    pub control_county: Option<String>,
}

impl ControlPointRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            control_point_action_performed: parse_opt_char(fields.get(5).unwrap_or(&"")),
            control_point_number: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            control_address: parse_opt_string(fields.get(7).unwrap_or(&"")),
            control_city: parse_opt_string(fields.get(8).unwrap_or(&"")),
            state_code: parse_opt_string(fields.get(9).unwrap_or(&"")),
            control_phone: parse_opt_string(fields.get(10).unwrap_or(&"")),
            control_county: parse_opt_string(fields.get(11).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_point_from_fields() {
        let fields = vec![
            "CP",
            "12345",
            "ULS123",
            "EBF456",
            "W1TEST",
            "A",
            "1",
            "123 Main St",
            "Springfield",
            "IL",
            "555-555-1234",
            "Sangamon",
        ];
        let cp = ControlPointRecord::from_fields(&fields);

        assert_eq!(cp.unique_system_identifier, 12345);
        assert_eq!(cp.call_sign, Some("W1TEST".to_string()));
        assert_eq!(cp.control_city, Some("Springfield".to_string()));
        assert_eq!(cp.state_code, Some("IL".to_string()));
    }
}
