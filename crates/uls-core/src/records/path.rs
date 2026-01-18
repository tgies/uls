//! PA (Path) record type for microwave links.
use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub callsign: Option<String>,
    pub path_action_performed: Option<char>,
    pub path_number: Option<i32>,
    pub transmit_location_number: Option<i32>,
    pub transmit_antenna_number: Option<i32>,
    pub receiver_location_number: Option<i32>,
    pub receiver_antenna_number: Option<i32>,
    pub path_type_desc: Option<String>,
    pub passive_receiver_indicator: Option<char>,
    pub country_code: Option<String>,
    pub receiver_callsign: Option<String>,
}

impl PathRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            path_action_performed: parse_opt_char(fields.get(5).unwrap_or(&"")),
            path_number: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            transmit_location_number: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            transmit_antenna_number: parse_opt_i32(fields.get(8).unwrap_or(&"")),
            receiver_location_number: parse_opt_i32(fields.get(9).unwrap_or(&"")),
            receiver_antenna_number: parse_opt_i32(fields.get(10).unwrap_or(&"")),
            path_type_desc: parse_opt_string(fields.get(11).unwrap_or(&"")),
            passive_receiver_indicator: parse_opt_char(fields.get(12).unwrap_or(&"")),
            country_code: parse_opt_string(fields.get(13).unwrap_or(&"")),
            receiver_callsign: parse_opt_string(fields.get(14).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_from_fields() {
        let fields = vec![
            "PA",
            "12345",
            "ULS123",
            "EBF456",
            "W1TEST",
            "A",
            "1",
            "1",
            "2",
            "2",
            "3",
            "MICROWAVE",
            "N",
            "US",
            "W2TEST",
        ];
        let pa = PathRecord::from_fields(&fields);

        assert_eq!(pa.unique_system_identifier, 12345);
        assert_eq!(pa.callsign, Some("W1TEST".to_string()));
        assert_eq!(pa.path_number, Some(1));
        assert_eq!(pa.path_type_desc, Some("MICROWAVE".to_string()));
        assert_eq!(pa.receiver_callsign, Some("W2TEST".to_string()));
    }
}
