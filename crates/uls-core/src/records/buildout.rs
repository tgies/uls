//! BO/BL/BF (Buildout) record types.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildoutRecord {
    pub unique_system_identifier: i64,
    pub call_sign: Option<String>,
    pub buildout_code: Option<i32>,
    pub buildout_deadline: Option<String>,
    pub buildout_date: Option<String>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
}

impl BuildoutRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(2).unwrap_or(&"")),
            buildout_code: parse_opt_i32(fields.get(3).unwrap_or(&"")),
            buildout_deadline: parse_opt_string(fields.get(4).unwrap_or(&"")),
            buildout_date: parse_opt_string(fields.get(5).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(6).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(7).unwrap_or(&"")),
        }
    }
}
