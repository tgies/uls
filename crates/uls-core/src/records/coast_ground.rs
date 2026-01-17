//! CG (Coast & Ground) record type.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoastGroundRecord {
    pub unique_system_identifier: Option<i64>,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub station_available: Option<char>,
    pub public_correspondence: Option<char>,
    pub station_identifier: Option<String>,
    pub station_class: Option<String>,
}

impl CoastGroundRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_opt_i64(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            station_available: parse_opt_char(fields.get(5).unwrap_or(&"")),
            public_correspondence: parse_opt_char(fields.get(6).unwrap_or(&"")),
            station_identifier: parse_opt_string(fields.get(7).unwrap_or(&"")),
            station_class: parse_opt_string(fields.get(40).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coast_ground_from_fields() {
        let fields = vec!["CG", "12345", "ULS123", "EBF456", "WLO", "Y", "Y", "STNID123"];
        let cg = CoastGroundRecord::from_fields(&fields);
        
        assert_eq!(cg.unique_system_identifier, Some(12345));
        assert_eq!(cg.call_sign, Some("WLO".to_string()));
        assert_eq!(cg.station_available, Some('Y'));
    }
}

