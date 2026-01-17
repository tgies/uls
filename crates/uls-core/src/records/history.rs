//! HS (History) and CO (Comment) record types.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub callsign: Option<String>,
    pub log_date: Option<String>,
    pub code: Option<String>,
}

impl HistoryRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(3).unwrap_or(&"")),
            log_date: parse_opt_string(fields.get(4).unwrap_or(&"")),
            code: parse_opt_string(fields.get(5).unwrap_or(&"")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRecord {
    pub unique_system_identifier: i64,
    pub uls_file_num: Option<String>,
    pub callsign: Option<String>,
    pub comment_date: Option<String>,
    pub description: Option<String>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
}

impl CommentRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_num: parse_opt_string(fields.get(2).unwrap_or(&"")),
            callsign: parse_opt_string(fields.get(3).unwrap_or(&"")),
            comment_date: parse_opt_string(fields.get(4).unwrap_or(&"")),
            description: parse_opt_string(fields.get(5).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(6).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(7).unwrap_or(&"")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDetailRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub application_purpose: Option<String>,
    pub application_status: Option<char>,
    pub application_fee_exempt: Option<char>,
    pub regulatory_fee_exempt: Option<char>,
    pub source: Option<char>,
    pub receipt_date: Option<String>,
    pub notification_code: Option<char>,
    pub notification_date: Option<String>,
}

impl ApplicationDetailRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            application_purpose: parse_opt_string(fields.get(4).unwrap_or(&"")),
            application_status: parse_opt_char(fields.get(5).unwrap_or(&"")),
            application_fee_exempt: parse_opt_char(fields.get(6).unwrap_or(&"")),
            regulatory_fee_exempt: parse_opt_char(fields.get(7).unwrap_or(&"")),
            source: parse_opt_char(fields.get(8).unwrap_or(&"")),
            receipt_date: parse_opt_string(fields.get(10).unwrap_or(&"")),
            notification_code: parse_opt_char(fields.get(11).unwrap_or(&"")),
            notification_date: parse_opt_string(fields.get(12).unwrap_or(&"")),
        }
    }
}
