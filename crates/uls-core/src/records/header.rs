//! HD (Header) record type - License/Application Header.
//!
//! The HD record contains the main form data that carries to the license.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::common::*;

/// HD record - License/Application Header.
/// This is the primary record for a license or application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRecord {
    // Required fields
    pub unique_system_identifier: i64,

    // Optional fields
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub license_status: Option<char>,
    pub radio_service_code: Option<String>,
    pub grant_date: Option<NaiveDate>,
    pub expired_date: Option<NaiveDate>,
    pub cancellation_date: Option<NaiveDate>,
    pub eligibility_rule_num: Option<String>,
    pub applicant_type_code_reserved: Option<char>,
    pub alien: Option<char>,
    pub alien_government: Option<char>,
    pub alien_corporation: Option<char>,
    pub alien_officer: Option<char>,
    pub alien_control: Option<char>,
    pub revoked: Option<char>,
    pub convicted: Option<char>,
    pub adjudged: Option<char>,
    pub involved_reserved: Option<char>,
    pub common_carrier: Option<char>,
    pub non_common_carrier: Option<char>,
    pub private_comm: Option<char>,
    pub fixed: Option<char>,
    pub mobile: Option<char>,
    pub radiolocation: Option<char>,
    pub satellite: Option<char>,
    pub developmental_or_sta: Option<char>,
    pub interconnected_service: Option<char>,
    pub certifier_first_name: Option<String>,
    pub certifier_mi: Option<char>,
    pub certifier_last_name: Option<String>,
    pub certifier_suffix: Option<String>,
    pub certifier_title: Option<String>,
    pub sex: Option<char>,
    pub african_american: Option<char>,
    pub native_american: Option<char>,
    pub hawaiian: Option<char>,
    pub asian: Option<char>,
    pub white: Option<char>,
    pub ethnicity: Option<char>,
    pub effective_date: Option<NaiveDate>,
    pub last_action_date: Option<NaiveDate>,
    pub auction_id: Option<i32>,
    pub reg_stat_broad_serv: Option<char>,
    pub band_manager: Option<char>,
    pub type_serv_broad_serv: Option<char>,
    pub alien_ruling: Option<char>,
    pub licensee_name_change: Option<char>,
    pub whitespace_ind: Option<char>,
    pub additional_cert_choice: Option<char>,
    pub additional_cert_answer: Option<char>,
    pub discontinuation_ind: Option<char>,
    pub regulatory_compliance_ind: Option<char>,
    pub eligibility_cert_900: Option<char>,
    pub transition_plan_cert_900: Option<char>,
    pub return_spectrum_cert_900: Option<char>,
    pub payment_cert_900: Option<char>,
}

impl HeaderRecord {
    /// Parse a header record from pipe-delimited fields.
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            license_status: parse_opt_char(fields.get(5).unwrap_or(&"")),
            radio_service_code: parse_opt_string(fields.get(6).unwrap_or(&"")),
            grant_date: parse_uls_date(fields.get(7).unwrap_or(&"")),
            expired_date: parse_uls_date(fields.get(8).unwrap_or(&"")),
            cancellation_date: parse_uls_date(fields.get(9).unwrap_or(&"")),
            eligibility_rule_num: parse_opt_string(fields.get(10).unwrap_or(&"")),
            applicant_type_code_reserved: parse_opt_char(fields.get(11).unwrap_or(&"")),
            alien: parse_opt_char(fields.get(12).unwrap_or(&"")),
            alien_government: parse_opt_char(fields.get(13).unwrap_or(&"")),
            alien_corporation: parse_opt_char(fields.get(14).unwrap_or(&"")),
            alien_officer: parse_opt_char(fields.get(15).unwrap_or(&"")),
            alien_control: parse_opt_char(fields.get(16).unwrap_or(&"")),
            revoked: parse_opt_char(fields.get(17).unwrap_or(&"")),
            convicted: parse_opt_char(fields.get(18).unwrap_or(&"")),
            adjudged: parse_opt_char(fields.get(19).unwrap_or(&"")),
            involved_reserved: parse_opt_char(fields.get(20).unwrap_or(&"")),
            common_carrier: parse_opt_char(fields.get(21).unwrap_or(&"")),
            non_common_carrier: parse_opt_char(fields.get(22).unwrap_or(&"")),
            private_comm: parse_opt_char(fields.get(23).unwrap_or(&"")),
            fixed: parse_opt_char(fields.get(24).unwrap_or(&"")),
            mobile: parse_opt_char(fields.get(25).unwrap_or(&"")),
            radiolocation: parse_opt_char(fields.get(26).unwrap_or(&"")),
            satellite: parse_opt_char(fields.get(27).unwrap_or(&"")),
            developmental_or_sta: parse_opt_char(fields.get(28).unwrap_or(&"")),
            interconnected_service: parse_opt_char(fields.get(29).unwrap_or(&"")),
            certifier_first_name: parse_opt_string(fields.get(30).unwrap_or(&"")),
            certifier_mi: parse_opt_char(fields.get(31).unwrap_or(&"")),
            certifier_last_name: parse_opt_string(fields.get(32).unwrap_or(&"")),
            certifier_suffix: parse_opt_string(fields.get(33).unwrap_or(&"")),
            certifier_title: parse_opt_string(fields.get(34).unwrap_or(&"")),
            sex: parse_opt_char(fields.get(35).unwrap_or(&"")),
            african_american: parse_opt_char(fields.get(36).unwrap_or(&"")),
            native_american: parse_opt_char(fields.get(37).unwrap_or(&"")),
            hawaiian: parse_opt_char(fields.get(38).unwrap_or(&"")),
            asian: parse_opt_char(fields.get(39).unwrap_or(&"")),
            white: parse_opt_char(fields.get(40).unwrap_or(&"")),
            ethnicity: parse_opt_char(fields.get(41).unwrap_or(&"")),
            effective_date: parse_uls_date(fields.get(42).unwrap_or(&"")),
            last_action_date: parse_uls_date(fields.get(43).unwrap_or(&"")),
            auction_id: parse_opt_i32(fields.get(44).unwrap_or(&"")),
            reg_stat_broad_serv: parse_opt_char(fields.get(45).unwrap_or(&"")),
            band_manager: parse_opt_char(fields.get(46).unwrap_or(&"")),
            type_serv_broad_serv: parse_opt_char(fields.get(47).unwrap_or(&"")),
            alien_ruling: parse_opt_char(fields.get(48).unwrap_or(&"")),
            licensee_name_change: parse_opt_char(fields.get(49).unwrap_or(&"")),
            whitespace_ind: parse_opt_char(fields.get(50).unwrap_or(&"")),
            additional_cert_choice: parse_opt_char(fields.get(51).unwrap_or(&"")),
            additional_cert_answer: parse_opt_char(fields.get(52).unwrap_or(&"")),
            discontinuation_ind: parse_opt_char(fields.get(53).unwrap_or(&"")),
            regulatory_compliance_ind: parse_opt_char(fields.get(54).unwrap_or(&"")),
            eligibility_cert_900: parse_opt_char(fields.get(55).unwrap_or(&"")),
            transition_plan_cert_900: parse_opt_char(fields.get(56).unwrap_or(&"")),
            return_spectrum_cert_900: parse_opt_char(fields.get(57).unwrap_or(&"")),
            payment_cert_900: parse_opt_char(fields.get(58).unwrap_or(&"")),
        }
    }

    /// Returns true if this is an active license.
    pub fn is_active(&self) -> bool {
        self.license_status == Some('A')
    }

    /// Returns true if this license has expired.
    pub fn is_expired(&self) -> bool {
        self.license_status == Some('E')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_from_fields() {
        let fields: Vec<&str> = vec![
            "HD",
            "123456789",
            "0000123456",
            "",
            "W1AW",
            "A",
            "HA",
            "01/01/2020",
            "01/01/2030",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "01/01/2020",
            "06/15/2024",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ];
        let record = HeaderRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 123456789);
        assert_eq!(record.call_sign, Some("W1AW".to_string()));
        assert_eq!(record.license_status, Some('A'));
        assert_eq!(record.radio_service_code, Some("HA".to_string()));
        assert!(record.is_active());
    }

    #[test]
    fn test_header_empty_fields() {
        let fields: Vec<&str> = vec!["HD", "", "", "", "", "", ""];
        let record = HeaderRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 0);
        assert_eq!(record.call_sign, None);
        assert!(!record.is_active());
    }
}
