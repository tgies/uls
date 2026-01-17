//! EM (Emission) record type.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub location_number: Option<i32>,
    pub antenna_number: Option<i32>,
    pub frequency_assigned: Option<f64>,
    pub emission_action_performed: Option<char>,
    pub emission_code: Option<String>,
    pub digital_mod_rate: Option<f64>,
    pub digital_mod_type: Option<String>,
    pub frequency_number: Option<i32>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
    pub emission_sequence_id: Option<i32>,
}

impl EmissionRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            location_number: parse_opt_i32(fields.get(5).unwrap_or(&"")),
            antenna_number: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            frequency_assigned: parse_opt_f64(fields.get(7).unwrap_or(&"")),
            emission_action_performed: parse_opt_char(fields.get(8).unwrap_or(&"")),
            emission_code: parse_opt_string(fields.get(9).unwrap_or(&"")),
            digital_mod_rate: parse_opt_f64(fields.get(10).unwrap_or(&"")),
            digital_mod_type: parse_opt_string(fields.get(11).unwrap_or(&"")),
            frequency_number: parse_opt_i32(fields.get(12).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(13).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(14).unwrap_or(&"")),
            emission_sequence_id: parse_opt_i32(fields.get(15).unwrap_or(&"")),
        }
    }
}
