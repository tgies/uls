//! FR (Frequency) record type - Frequency assignment data.

use serde::{Deserialize, Serialize};

use super::common::*;

/// FR record - Frequency data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub frequency_action_performed: Option<char>,
    pub location_number: Option<i32>,
    pub antenna_number: Option<i32>,
    pub class_station_code: Option<String>,
    pub op_altitude_code: Option<String>,
    pub frequency_assigned: Option<f64>,
    pub frequency_upper_band: Option<f64>,
    pub frequency_carrier: Option<f64>,
    pub time_begin_operations: Option<i32>,
    pub time_end_operations: Option<i32>,
    pub power_output: Option<f64>,
    pub power_erp: Option<f64>,
    pub tolerance: Option<f64>,
    pub frequency_ind: Option<char>,
    pub status: Option<char>,
    pub eirp: Option<f64>,
    pub transmitter_make: Option<String>,
    pub transmitter_model: Option<String>,
    pub auto_transmitter_power_control: Option<char>,
    pub cnt_mobile_units: Option<i32>,
    pub cnt_mob_pagers: Option<i32>,
    pub freq_seq_id: Option<i32>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
    pub date_first_used: Option<String>,
}

impl FrequencyRecord {
    /// Parse a frequency record from pipe-delimited fields.
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            frequency_action_performed: parse_opt_char(fields.get(5).unwrap_or(&"")),
            location_number: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            antenna_number: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            class_station_code: parse_opt_string(fields.get(8).unwrap_or(&"")),
            op_altitude_code: parse_opt_string(fields.get(9).unwrap_or(&"")),
            frequency_assigned: parse_opt_f64(fields.get(10).unwrap_or(&"")),
            frequency_upper_band: parse_opt_f64(fields.get(11).unwrap_or(&"")),
            frequency_carrier: parse_opt_f64(fields.get(12).unwrap_or(&"")),
            time_begin_operations: parse_opt_i32(fields.get(13).unwrap_or(&"")),
            time_end_operations: parse_opt_i32(fields.get(14).unwrap_or(&"")),
            power_output: parse_opt_f64(fields.get(15).unwrap_or(&"")),
            power_erp: parse_opt_f64(fields.get(16).unwrap_or(&"")),
            tolerance: parse_opt_f64(fields.get(17).unwrap_or(&"")),
            frequency_ind: parse_opt_char(fields.get(18).unwrap_or(&"")),
            status: parse_opt_char(fields.get(19).unwrap_or(&"")),
            eirp: parse_opt_f64(fields.get(20).unwrap_or(&"")),
            transmitter_make: parse_opt_string(fields.get(21).unwrap_or(&"")),
            transmitter_model: parse_opt_string(fields.get(22).unwrap_or(&"")),
            auto_transmitter_power_control: parse_opt_char(fields.get(23).unwrap_or(&"")),
            cnt_mobile_units: parse_opt_i32(fields.get(24).unwrap_or(&"")),
            cnt_mob_pagers: parse_opt_i32(fields.get(25).unwrap_or(&"")),
            freq_seq_id: parse_opt_i32(fields.get(26).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(27).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(28).unwrap_or(&"")),
            date_first_used: parse_opt_string(fields.get(29).unwrap_or(&"")),
        }
    }

    /// Returns the frequency in MHz.
    pub fn frequency_mhz(&self) -> Option<f64> {
        self.frequency_assigned
    }

    /// Returns the frequency in kHz.
    pub fn frequency_khz(&self) -> Option<f64> {
        self.frequency_assigned.map(|f| f * 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_from_fields() {
        let fields: Vec<&str> = vec![
            "FR",
            "123456789",
            "",
            "",
            "W1AW",
            "",
            "1",
            "1",
            "FB",
            "",
            "146.94000000",
            "",
            "",
            "0",
            "2400",
            "50.0",
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
        let record = FrequencyRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 123456789);
        assert!((record.frequency_assigned.unwrap() - 146.94).abs() < 0.001);
    }
}
