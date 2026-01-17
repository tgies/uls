//! AC (Aircraft) record type.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub aircraft_count: Option<i32>,
    pub type_of_carrier: Option<char>,
    pub portable_indicator: Option<char>,
    pub fleet_indicator: Option<char>,
    pub n_number: Option<String>,
}

impl AircraftRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            aircraft_count: parse_opt_i32(fields.get(5).unwrap_or(&"")),
            type_of_carrier: parse_opt_char(fields.get(6).unwrap_or(&"")),
            portable_indicator: parse_opt_char(fields.get(7).unwrap_or(&"")),
            fleet_indicator: parse_opt_char(fields.get(8).unwrap_or(&"")),
            n_number: parse_opt_string(fields.get(9).unwrap_or(&"")),
        }
    }
}
