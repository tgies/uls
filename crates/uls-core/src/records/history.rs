//! HS (History) and CO (Comment) record types.
use super::common::*;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_from_fields() {
        let fields = vec!["HS", "12345", "ULS123", "W1TEST", "01/01/2020", "LIISS"];
        let hs = HistoryRecord::from_fields(&fields);

        assert_eq!(hs.unique_system_identifier, 12345);
        assert_eq!(hs.uls_file_number, Some("ULS123".to_string()));
        assert_eq!(hs.callsign, Some("W1TEST".to_string()));
        assert_eq!(hs.log_date, Some("01/01/2020".to_string()));
        assert_eq!(hs.code, Some("LIISS".to_string()));
    }

    #[test]
    fn test_comment_from_fields() {
        let fields = vec![
            "CO",
            "12345",
            "ULS123",
            "W1TEST",
            "01/01/2020",
            "This is a test comment",
            "A",
            "01/01/2020",
        ];
        let co = CommentRecord::from_fields(&fields);

        assert_eq!(co.unique_system_identifier, 12345);
        assert_eq!(co.callsign, Some("W1TEST".to_string()));
        assert_eq!(co.description, Some("This is a test comment".to_string()));
        assert_eq!(co.status_code, Some('A'));
    }

    #[test]
    fn test_application_detail_from_fields() {
        let fields = vec![
            "AD",
            "12345",
            "ULS123",
            "EBF456",
            "NE",
            "G",
            "N",
            "N",
            "E",
            "",
            "01/01/2020",
            "A",
            "01/02/2020",
        ];
        let ad = ApplicationDetailRecord::from_fields(&fields);

        assert_eq!(ad.unique_system_identifier, 12345);
        assert_eq!(ad.application_purpose, Some("NE".to_string()));
        assert_eq!(ad.application_status, Some('G'));
        assert_eq!(ad.source, Some('E'));
        assert_eq!(ad.receipt_date, Some("01/01/2020".to_string()));
        assert_eq!(ad.notification_code, Some('A'));
    }
}
