//! SH (Ship) record type.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipRecord {
    pub unique_system_identifier: Option<i64>,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub callsign: Option<String>,
    pub type_of_authorization: Option<char>,
    pub count_in_fleet: Option<i32>,
    pub general_class: Option<String>,
    pub special_class: Option<String>,
    pub ship_name: Option<String>,
    pub ship_number: Option<String>,
    pub international_voyages: Option<char>,
    pub foreign_communications: Option<char>,
    pub radiotelegraph: Option<char>,
    pub mmsi_request: Option<char>,
    pub gross_tonnage: Option<i32>,
    pub ship_length: Option<i32>,
}

impl ShipRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_opt_i64(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            type_of_authorization: parse_opt_char(fields.get(5).unwrap_or(&"")),
            count_in_fleet: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            general_class: parse_opt_string(fields.get(7).unwrap_or(&"")),
            special_class: parse_opt_string(fields.get(8).unwrap_or(&"")),
            ship_name: parse_opt_string(fields.get(9).unwrap_or(&"")),
            ship_number: parse_opt_string(fields.get(10).unwrap_or(&"")),
            international_voyages: parse_opt_char(fields.get(11).unwrap_or(&"")),
            foreign_communications: parse_opt_char(fields.get(12).unwrap_or(&"")),
            radiotelegraph: parse_opt_char(fields.get(13).unwrap_or(&"")),
            mmsi_request: parse_opt_char(fields.get(14).unwrap_or(&"")),
            gross_tonnage: parse_opt_i32(fields.get(15).unwrap_or(&"")),
            ship_length: parse_opt_i32(fields.get(16).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_from_fields() {
        let fields = vec![
            "SH", "12345", "ULS123", "EBF456", "WDA1234", "S", "5",
            "CARGO", "TANKER", "SS MINNOW", "IMO12345", "Y", "Y", "N", "Y", "5000", "100",
        ];
        let sh = ShipRecord::from_fields(&fields);
        
        assert_eq!(sh.unique_system_identifier, Some(12345));
        assert_eq!(sh.callsign, Some("WDA1234".to_string()));
        assert_eq!(sh.ship_name, Some("SS MINNOW".to_string()));
        assert_eq!(sh.gross_tonnage, Some(5000));
        assert_eq!(sh.ship_length, Some(100));
    }
}

