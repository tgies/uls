//! LO (Location) record type - Site/location data.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::common::*;

/// LO record - Location data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRecord {
    pub unique_system_identifier: i64,
    pub uls_file_number: Option<String>,
    pub ebf_number: Option<String>,
    pub call_sign: Option<String>,
    pub location_action_performed: Option<char>,
    pub location_type_code: Option<char>,
    pub location_class_code: Option<char>,
    pub location_number: Option<i32>,
    pub site_status: Option<char>,
    pub corresponding_fixed_location: Option<i32>,
    pub location_address: Option<String>,
    pub location_city: Option<String>,
    pub location_county: Option<String>,
    pub location_state: Option<String>,
    pub radius_of_operation: Option<f64>,
    pub area_of_operation_code: Option<char>,
    pub clearance_indicator: Option<char>,
    pub ground_elevation: Option<f64>,
    pub coordinates: Coordinates,
    pub max_coordinates: Coordinates,
    pub nepa: Option<char>,
    pub quiet_zone_notification_date: Option<NaiveDate>,
    pub tower_registration_number: Option<String>,
    pub height_of_support_structure: Option<f64>,
    pub overall_height_of_structure: Option<f64>,
    pub structure_type: Option<String>,
    pub airport_id: Option<String>,
    pub location_name: Option<String>,
    pub units_hand_held: Option<i32>,
    pub units_mobile: Option<i32>,
    pub units_temp_fixed: Option<i32>,
    pub units_aircraft: Option<i32>,
    pub units_itinerant: Option<i32>,
    pub status_code: Option<char>,
    pub status_date: Option<String>,
    pub earth_agree: Option<char>,
}

impl LocationRecord {
    /// Parse a location record from pipe-delimited fields.
    pub fn from_fields(fields: &[&str]) -> Self {
        Self {
            unique_system_identifier: parse_i64_or_default(fields.get(1).unwrap_or(&"")),
            uls_file_number: parse_opt_string(fields.get(2).unwrap_or(&"")),
            ebf_number: parse_opt_string(fields.get(3).unwrap_or(&"")),
            call_sign: parse_opt_string(fields.get(4).unwrap_or(&"")),
            location_action_performed: parse_opt_char(fields.get(5).unwrap_or(&"")),
            location_type_code: parse_opt_char(fields.get(6).unwrap_or(&"")),
            location_class_code: parse_opt_char(fields.get(7).unwrap_or(&"")),
            location_number: parse_opt_i32(fields.get(8).unwrap_or(&"")),
            site_status: parse_opt_char(fields.get(9).unwrap_or(&"")),
            corresponding_fixed_location: parse_opt_i32(fields.get(10).unwrap_or(&"")),
            location_address: parse_opt_string(fields.get(11).unwrap_or(&"")),
            location_city: parse_opt_string(fields.get(12).unwrap_or(&"")),
            location_county: parse_opt_string(fields.get(13).unwrap_or(&"")),
            location_state: parse_opt_string(fields.get(14).unwrap_or(&"")),
            radius_of_operation: parse_opt_f64(fields.get(15).unwrap_or(&"")),
            area_of_operation_code: parse_opt_char(fields.get(16).unwrap_or(&"")),
            clearance_indicator: parse_opt_char(fields.get(17).unwrap_or(&"")),
            ground_elevation: parse_opt_f64(fields.get(18).unwrap_or(&"")),
            coordinates: Coordinates {
                lat_degrees: parse_opt_i32(fields.get(19).unwrap_or(&"")),
                lat_minutes: parse_opt_i32(fields.get(20).unwrap_or(&"")),
                lat_seconds: parse_opt_f64(fields.get(21).unwrap_or(&"")),
                lat_direction: parse_opt_char(fields.get(22).unwrap_or(&"")),
                long_degrees: parse_opt_i32(fields.get(23).unwrap_or(&"")),
                long_minutes: parse_opt_i32(fields.get(24).unwrap_or(&"")),
                long_seconds: parse_opt_f64(fields.get(25).unwrap_or(&"")),
                long_direction: parse_opt_char(fields.get(26).unwrap_or(&"")),
            },
            max_coordinates: Coordinates {
                lat_degrees: parse_opt_i32(fields.get(27).unwrap_or(&"")),
                lat_minutes: parse_opt_i32(fields.get(28).unwrap_or(&"")),
                lat_seconds: parse_opt_f64(fields.get(29).unwrap_or(&"")),
                lat_direction: parse_opt_char(fields.get(30).unwrap_or(&"")),
                long_degrees: parse_opt_i32(fields.get(31).unwrap_or(&"")),
                long_minutes: parse_opt_i32(fields.get(32).unwrap_or(&"")),
                long_seconds: parse_opt_f64(fields.get(33).unwrap_or(&"")),
                long_direction: parse_opt_char(fields.get(34).unwrap_or(&"")),
            },
            nepa: parse_opt_char(fields.get(35).unwrap_or(&"")),
            quiet_zone_notification_date: parse_uls_date(fields.get(36).unwrap_or(&"")),
            tower_registration_number: parse_opt_string(fields.get(37).unwrap_or(&"")),
            height_of_support_structure: parse_opt_f64(fields.get(38).unwrap_or(&"")),
            overall_height_of_structure: parse_opt_f64(fields.get(39).unwrap_or(&"")),
            structure_type: parse_opt_string(fields.get(40).unwrap_or(&"")),
            airport_id: parse_opt_string(fields.get(41).unwrap_or(&"")),
            location_name: parse_opt_string(fields.get(42).unwrap_or(&"")),
            units_hand_held: parse_opt_i32(fields.get(43).unwrap_or(&"")),
            units_mobile: parse_opt_i32(fields.get(44).unwrap_or(&"")),
            units_temp_fixed: parse_opt_i32(fields.get(45).unwrap_or(&"")),
            units_aircraft: parse_opt_i32(fields.get(46).unwrap_or(&"")),
            units_itinerant: parse_opt_i32(fields.get(47).unwrap_or(&"")),
            status_code: parse_opt_char(fields.get(48).unwrap_or(&"")),
            status_date: parse_opt_string(fields.get(49).unwrap_or(&"")),
            earth_agree: parse_opt_char(fields.get(50).unwrap_or(&"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_from_fields() {
        // Indices: 0=type, 1=usi, 2=uls_file, 3=ebf, 4=callsign, 5=action, 6=loc_type, 7=loc_class,
        //          8=loc_number, 9=site_status, 10=fixed_loc, 11=address, 12=city, 13=county, 14=state,
        //          15=radius, 16=area_code, 17=clearance, 18=ground_elev,
        //          19=lat_deg, 20=lat_min, 21=lat_sec, 22=lat_dir, 23=long_deg, 24=long_min, 25=long_sec, 26=long_dir
        let fields: Vec<&str> = vec![
            "LO",
            "123456789",
            "",
            "",
            "W1AW",
            "",
            "F",
            "F",
            "1",
            "A",
            "",
            "225 MAIN ST",
            "NEWINGTON",
            "HARTFORD",
            "CT",
            "",
            "",
            "",
            "",
            "41",
            "43",
            "12.3",
            "N",
            "72",
            "44",
            "30.1",
            "W",
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
        ];
        let record = LocationRecord::from_fields(&fields);
        assert_eq!(record.unique_system_identifier, 123456789);
        assert_eq!(record.location_city, Some("NEWINGTON".to_string()));
        assert_eq!(record.coordinates.lat_degrees, Some(41));
        assert_eq!(record.coordinates.lat_minutes, Some(43));
        assert_eq!(record.coordinates.long_degrees, Some(72));
    }
}
