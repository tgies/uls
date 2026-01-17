//! TA (Transfer/Assignment) record type.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub pro_forma: Option<char>,
    pub full_assignment: Option<char>,
    pub voluntary_involuntary: Option<char>,
    pub consent_date: Option<String>,
    pub consummation_date: Option<String>,
    pub consummation_deadline: Option<String>,
}

impl TransferRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            pro_forma: parse_opt_char(fields.get(4).unwrap_or(&"")),
            full_assignment: parse_opt_char(fields.get(5).unwrap_or(&"")),
            voluntary_involuntary: parse_opt_char(fields.get(8).unwrap_or(&"")),
            consent_date: parse_opt_string(fields.get(30).unwrap_or(&"")),
            consummation_date: parse_opt_string(fields.get(31).unwrap_or(&"")),
            consummation_deadline: parse_opt_string(fields.get(32).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_from_fields() {
        let fields = vec!["TA", "12345", "ULS123", "EBF456", "Y", "Y", "", "", "V"];
        let ta = TransferRecord::from_fields(&fields);
        
        assert_eq!(ta.unique_system_identifier, 12345);
        assert_eq!(ta.pro_forma, Some('Y'));
        assert_eq!(ta.full_assignment, Some('Y'));
        assert_eq!(ta.voluntary_involuntary, Some('V'));
    }
}

