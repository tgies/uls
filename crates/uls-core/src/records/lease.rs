//! Lease-related record types (LC, LD, LL, L3-L6).
use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub lease_id: Option<String>,
}

impl LeaseRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            lease_id: parse_opt_string(fields.get(5).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lease_from_fields() {
        let fields = vec!["LC", "12345", "ULS123", "EBF456", "W1TEST", "LEASE001"];
        let lc = LeaseRecord::from_fields(&fields);

        assert_eq!(lc.unique_system_identifier, 12345);
        assert_eq!(lc.call_sign, Some("W1TEST".to_string()));
        assert_eq!(lc.lease_id, Some("LEASE001".to_string()));
    }
}
