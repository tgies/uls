//! Market-based license record types (MK, MP, MF, MC).
use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub market_code: Option<String>,
    pub channel_block: Option<String>,
    pub submarket_code: Option<i32>,
    pub market_name: Option<String>,
    pub coverage_partitioning: Option<char>,
    pub coverage_disaggregation: Option<char>,
    pub cellular_phase_id: Option<i32>,
    pub population: Option<i64>,
}

impl MarketRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            market_code: parse_opt_string(fields.get(5).unwrap_or(&"")),
            channel_block: parse_opt_string(fields.get(6).unwrap_or(&"")),
            submarket_code: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            market_name: parse_opt_string(fields.get(8).unwrap_or(&"")),
            coverage_partitioning: parse_opt_char(fields.get(9).unwrap_or(&"")),
            coverage_disaggregation: parse_opt_char(fields.get(10).unwrap_or(&"")),
            cellular_phase_id: parse_opt_i32(fields.get(11).unwrap_or(&"")),
            population: parse_opt_i64(fields.get(12).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_from_fields() {
        let fields = vec![
            "MK", "12345", "ULS123", "EBF456", "W1TEST", "MKT001", "A", "1", "New York", "Y", "N",
            "5", "8000000",
        ];
        let mk = MarketRecord::from_fields(&fields);

        assert_eq!(mk.unique_system_identifier, 12345);
        assert_eq!(mk.call_sign, Some("W1TEST".to_string()));
        assert_eq!(mk.market_code, Some("MKT001".to_string()));
        assert_eq!(mk.market_name, Some("New York".to_string()));
        assert_eq!(mk.population, Some(8000000));
    }
}
