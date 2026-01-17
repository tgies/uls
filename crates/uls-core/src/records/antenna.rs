//! AN (Antenna) record type - Antenna specifications.

use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntennaRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub antenna_action_performed: Option<char>,
    pub antenna_number: Option<i32>,
    pub location_number: Option<i32>,
    pub receive_zone_code: Option<String>,
    pub antenna_type_code: Option<char>,
    pub height_to_tip: Option<f64>,
    pub height_to_center_raat: Option<f64>,
    pub antenna_make: Option<String>,
    pub antenna_model: Option<String>,
    pub tilt: Option<f64>,
    pub polarization_code: Option<String>,
    pub beamwidth: Option<f64>,
    pub gain: Option<f64>,
    pub azimuth: Option<f64>,
    pub height_above_avg_terrain: Option<f64>,
    pub diversity_height: Option<f64>,
    pub diversity_gain: Option<f64>,
    pub diversity_beam: Option<f64>,
    pub reflector_height: Option<f64>,
    pub reflector_width: Option<f64>,
    pub reflector_separation: Option<f64>,
    pub repeater_seq_num: Option<i32>,
    pub back_to_back_tx_dish_gain: Option<f64>,
    pub back_to_back_rx_dish_gain: Option<f64>,
    pub location_name: Option<String>,
    pub passive_repeater_id: Option<i32>,
    pub alternative_cgsa_method: Option<char>,
    pub path_number: Option<i32>,
    pub line_loss: Option<f64>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
    pub psd_nonpsd_methodology: Option<String>,
    pub maximum_erp: Option<f64>,
}

impl AntennaRecord {
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            antenna_action_performed: parse_opt_char(fields.get(5).unwrap_or(&"")),
            antenna_number: parse_opt_i32(fields.get(6).unwrap_or(&"")),
            location_number: parse_opt_i32(fields.get(7).unwrap_or(&"")),
            receive_zone_code: parse_opt_string(fields.get(8).unwrap_or(&"")),
            antenna_type_code: parse_opt_char(fields.get(9).unwrap_or(&"")),
            height_to_tip: parse_opt_f64(fields.get(10).unwrap_or(&"")),
            height_to_center_raat: parse_opt_f64(fields.get(11).unwrap_or(&"")),
            antenna_make: parse_opt_string(fields.get(12).unwrap_or(&"")),
            antenna_model: parse_opt_string(fields.get(13).unwrap_or(&"")),
            tilt: parse_opt_f64(fields.get(14).unwrap_or(&"")),
            polarization_code: parse_opt_string(fields.get(15).unwrap_or(&"")),
            beamwidth: parse_opt_f64(fields.get(16).unwrap_or(&"")),
            gain: parse_opt_f64(fields.get(17).unwrap_or(&"")),
            azimuth: parse_opt_f64(fields.get(18).unwrap_or(&"")),
            height_above_avg_terrain: parse_opt_f64(fields.get(19).unwrap_or(&"")),
            diversity_height: parse_opt_f64(fields.get(20).unwrap_or(&"")),
            diversity_gain: parse_opt_f64(fields.get(21).unwrap_or(&"")),
            diversity_beam: parse_opt_f64(fields.get(22).unwrap_or(&"")),
            reflector_height: parse_opt_f64(fields.get(23).unwrap_or(&"")),
            reflector_width: parse_opt_f64(fields.get(24).unwrap_or(&"")),
            reflector_separation: parse_opt_f64(fields.get(25).unwrap_or(&"")),
            repeater_seq_num: parse_opt_i32(fields.get(26).unwrap_or(&"")),
            back_to_back_tx_dish_gain: parse_opt_f64(fields.get(27).unwrap_or(&"")),
            back_to_back_rx_dish_gain: parse_opt_f64(fields.get(28).unwrap_or(&"")),
            location_name: parse_opt_string(fields.get(29).unwrap_or(&"")),
            passive_repeater_id: parse_opt_i32(fields.get(30).unwrap_or(&"")),
            alternative_cgsa_method: parse_opt_char(fields.get(31).unwrap_or(&"")),
            path_number: parse_opt_i32(fields.get(32).unwrap_or(&"")),
            line_loss: parse_opt_f64(fields.get(33).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(34).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(35).unwrap_or(&"")),
            psd_nonpsd_methodology: parse_opt_string(fields.get(36).unwrap_or(&"")),
            maximum_erp: parse_opt_f64(fields.get(37).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antenna_from_fields() {
        let fields = vec![
            "AN", "12345", "ULS123", "EBF456", "W1TEST", "A", "1", "2", "ZN1", "D",
            "100.5", "50.0", "Yagi", "YA-1000", "15.0", "H", "60.0", "12.5", "180.0",
        ];
        let an = AntennaRecord::from_fields(&fields);
        
        assert_eq!(an.unique_system_identifier, 12345);
        assert_eq!(an.call_sign, Some("W1TEST".to_string()));
        assert_eq!(an.antenna_number, Some(1));
        assert_eq!(an.height_to_tip, Some(100.5));
        assert_eq!(an.antenna_make, Some("Yagi".to_string()));
        assert_eq!(an.gain, Some(12.5));
    }
}

