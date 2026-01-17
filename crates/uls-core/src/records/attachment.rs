//! AT/AH/LA (Attachment) record types.
use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub attachment_code: Option<String>,
    pub attachment_description: Option<String>,
    pub attachment_date: Option<String>,
    pub attachment_file_name: Option<String>,
    pub attachment_action_performed: Option<char>,
}

impl AttachmentRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            attachment_code: parse_opt_string(fields.get(4).unwrap_or(&"")),
            attachment_description: parse_opt_string(fields.get(5).unwrap_or(&"")),
            attachment_date: parse_opt_string(fields.get(6).unwrap_or(&"")),
            attachment_file_name: parse_opt_string(fields.get(7).unwrap_or(&"")),
            attachment_action_performed: parse_opt_char(fields.get(8).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_from_fields() {
        let fields = vec!["LA", "12345", "ULS123", "EBF456", "LETTER", "Formal letter attachment", "01/01/2020", "letter.pdf", "A"];
        let at = AttachmentRecord::from_fields(&fields);
        
        assert_eq!(at.unique_system_identifier, 12345);
        assert_eq!(at.attachment_code, Some("LETTER".to_string()));
        assert_eq!(at.attachment_file_name, Some("letter.pdf".to_string()));
    }
}
