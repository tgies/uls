//! EN (Entity) record type - Licensee/Applicant information.

use serde::{Deserialize, Serialize};

use super::common::*;

/// EN record - Entity (licensee/applicant names and addresses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub entity_type: Option<String>,
    pub licensee_id: Option<String>,
    pub entity_name: Option<String>,
    pub first_name: Option<String>,
    pub mi: Option<char>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub po_box: Option<String>,
    pub attention_line: Option<String>,
    pub sgin: Option<String>,
    pub frn: Option<String>,
    pub applicant_type_code: Option<char>,
    pub applicant_type_other: Option<String>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
    pub lic_category_code: Option<char>,
    pub linked_license_id: Option<i64>,
    pub linked_callsign: Option<String>,
}

impl EntityRecord {
    /// Parse an entity record from pipe-delimited fields.
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            entity_type: parse_opt_string(fields.get(5).unwrap_or(&"")),
            licensee_id: parse_opt_string(fields.get(6).unwrap_or(&"")),
            entity_name: parse_opt_string(fields.get(7).unwrap_or(&"")),
            first_name: parse_opt_string(fields.get(8).unwrap_or(&"")),
            mi: parse_opt_char(fields.get(9).unwrap_or(&"")),
            last_name: parse_opt_string(fields.get(10).unwrap_or(&"")),
            suffix: parse_opt_string(fields.get(11).unwrap_or(&"")),
            phone: parse_opt_string(fields.get(12).unwrap_or(&"")),
            fax: parse_opt_string(fields.get(13).unwrap_or(&"")),
            email: parse_opt_string(fields.get(14).unwrap_or(&"")),
            street_address: parse_opt_string(fields.get(15).unwrap_or(&"")),
            city: parse_opt_string(fields.get(16).unwrap_or(&"")),
            state: parse_opt_string(fields.get(17).unwrap_or(&"")),
            zip_code: parse_opt_string(fields.get(18).unwrap_or(&"")),
            po_box: parse_opt_string(fields.get(19).unwrap_or(&"")),
            attention_line: parse_opt_string(fields.get(20).unwrap_or(&"")),
            sgin: parse_opt_string(fields.get(21).unwrap_or(&"")),
            frn: parse_opt_string(fields.get(22).unwrap_or(&"")),
            applicant_type_code: parse_opt_char(fields.get(23).unwrap_or(&"")),
            applicant_type_other: parse_opt_string(fields.get(24).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(25).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(26).unwrap_or(&"")),
            lic_category_code: parse_opt_char(fields.get(27).unwrap_or(&"")),
            linked_license_id: parse_opt_i64(fields.get(28).unwrap_or(&"")),
            linked_callsign: parse_opt_string(fields.get(29).unwrap_or(&"")),
        }
    }

    /// Returns the full name of the entity.
    pub fn full_name(&self) -> String {
        if let Some(ref entity_name) = self.entity_name {
            if !entity_name.is_empty() {
                return entity_name.clone();
            }
        }

        let mut parts = Vec::new();
        if let Some(ref first) = self.first_name {
            parts.push(first.as_str());
        }
        if let Some(mi) = self.mi {
            parts.push(Box::leak(mi.to_string().into_boxed_str()));
        }
        if let Some(ref last) = self.last_name {
            parts.push(last.as_str());
        }
        if let Some(ref suffix) = self.suffix {
            parts.push(suffix.as_str());
        }
        parts.join(" ")
    }

    /// Returns the full address as a single string.
    pub fn full_address(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref addr) = self.street_address {
            parts.push(addr.clone());
        }
        if let Some(ref city) = self.city {
            let city_state_zip = format!(
                "{}, {} {}",
                city,
                self.state.as_deref().unwrap_or(""),
                self.zip_code.as_deref().unwrap_or("")
            );
            parts.push(city_state_zip.trim().to_string());
        }
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_from_fields() {
        let fields: Vec<&str> = vec![
            "EN",
            "123456789",
            "",
            "",
            "W1AW",
            "L",
            "L00001",
            "",
            "HIRAM",
            "P",
            "MAXIM",
            "JR",
            "8601234567",
            "",
            "test@example.com",
            "225 MAIN ST",
            "NEWINGTON",
            "CT",
            "06111",
            "",
            "",
            "",
            "0001234567",
            "I",
            "",
            "",
            "",
            "",
            "",
            "",
        ];
        let record = EntityRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 123456789);
        assert_eq!(record.call_sign, Some("W1AW".to_string()));
        assert_eq!(record.first_name, Some("HIRAM".to_string()));
        assert_eq!(record.last_name, Some("MAXIM".to_string()));
        assert_eq!(record.city, Some("NEWINGTON".to_string()));
        assert_eq!(record.state, Some("CT".to_string()));
    }

    #[test]
    fn test_full_name() {
        let mut record = EntityRecord::from_fields(&["EN", "123"]);
        record.first_name = Some("John".to_string());
        record.last_name = Some("Doe".to_string());
        assert_eq!(record.full_name(), "John Doe");

        record.entity_name = Some("ACME Corp".to_string());
        assert_eq!(record.full_name(), "ACME Corp");
    }
}
