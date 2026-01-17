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
