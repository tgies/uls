//! Common types used across ULS records.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Geographic coordinates in degrees/minutes/seconds format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat_degrees: Option<i32>,
    pub lat_minutes: Option<i32>,
    pub lat_seconds: Option<f64>,
    pub lat_direction: Option<char>,
    pub long_degrees: Option<i32>,
    pub long_minutes: Option<i32>,
    pub long_seconds: Option<f64>,
    pub long_direction: Option<char>,
}

impl Coordinates {
    /// Creates empty coordinates.
    pub fn empty() -> Self {
        Self {
            lat_degrees: None,
            lat_minutes: None,
            lat_seconds: None,
            lat_direction: None,
            long_degrees: None,
            long_minutes: None,
            long_seconds: None,
            long_direction: None,
        }
    }

    /// Returns true if all coordinate fields are empty.
    pub fn is_empty(&self) -> bool {
        self.lat_degrees.is_none()
            && self.lat_minutes.is_none()
            && self.lat_seconds.is_none()
            && self.long_degrees.is_none()
            && self.long_minutes.is_none()
            && self.long_seconds.is_none()
    }

    /// Converts to decimal degrees if all required fields are present.
    pub fn to_decimal(&self) -> Option<(f64, f64)> {
        let lat_deg = self.lat_degrees?;
        let lat_min = self.lat_minutes.unwrap_or(0);
        let lat_sec = self.lat_seconds.unwrap_or(0.0);
        let lat_dir = self.lat_direction.unwrap_or('N');

        let long_deg = self.long_degrees?;
        let long_min = self.long_minutes.unwrap_or(0);
        let long_sec = self.long_seconds.unwrap_or(0.0);
        let long_dir = self.long_direction.unwrap_or('W');

        let mut lat = lat_deg as f64 + (lat_min as f64 / 60.0) + (lat_sec / 3600.0);
        let mut long = long_deg as f64 + (long_min as f64 / 60.0) + (long_sec / 3600.0);

        if lat_dir == 'S' {
            lat = -lat;
        }
        if long_dir == 'W' {
            long = -long;
        }

        Some((lat, long))
    }
}

impl Default for Coordinates {
    fn default() -> Self {
        Self::empty()
    }
}

/// Parse a ULS date string (MM/DD/YYYY format) into a NaiveDate.
pub fn parse_uls_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try MM/DD/YYYY format first
    if let Ok(date) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Some(date);
    }

    // Try YYYY-MM-DD format
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(date);
    }

    None
}

/// Parse an optional string field, returning None for empty strings.
pub fn parse_opt_string(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Parse an optional i32 field.
pub fn parse_opt_i32(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Parse an optional i64 field.
pub fn parse_opt_i64(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Parse an optional f64 field.
pub fn parse_opt_f64(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Parse an optional char field.
pub fn parse_opt_char(s: &str) -> Option<char> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.chars().next()
    }
}

/// Parse a required i64 field, returning 0 if empty/invalid.
pub fn parse_i64_or_default(s: &str) -> i64 {
    parse_opt_i64(s).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates_decimal() {
        let coords = Coordinates {
            lat_degrees: Some(40),
            lat_minutes: Some(30),
            lat_seconds: Some(0.0),
            lat_direction: Some('N'),
            long_degrees: Some(74),
            long_minutes: Some(0),
            long_seconds: Some(0.0),
            long_direction: Some('W'),
        };

        let (lat, long) = coords.to_decimal().unwrap();
        assert!((lat - 40.5).abs() < 0.001);
        assert!((long - (-74.0)).abs() < 0.001);
    }

    #[test]
    fn test_coordinates_empty() {
        let coords = Coordinates::empty();
        assert!(coords.is_empty());
        assert!(coords.to_decimal().is_none());
    }

    #[test]
    fn test_parse_uls_date() {
        assert_eq!(
            parse_uls_date("01/15/2024"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(
            parse_uls_date("2024-01-15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(parse_uls_date(""), None);
        assert_eq!(parse_uls_date("invalid"), None);
    }

    #[test]
    fn test_parse_opt_string() {
        assert_eq!(parse_opt_string("hello"), Some("hello".to_string()));
        assert_eq!(parse_opt_string("  hello  "), Some("hello".to_string()));
        assert_eq!(parse_opt_string(""), None);
        assert_eq!(parse_opt_string("   "), None);
    }

    #[test]
    fn test_parse_opt_i32() {
        assert_eq!(parse_opt_i32("123"), Some(123));
        assert_eq!(parse_opt_i32("-456"), Some(-456));
        assert_eq!(parse_opt_i32(""), None);
        assert_eq!(parse_opt_i32("abc"), None);
    }

    #[test]
    fn test_parse_opt_f64() {
        assert_eq!(parse_opt_f64("123.456"), Some(123.456));
        assert_eq!(parse_opt_f64("-0.5"), Some(-0.5));
        assert_eq!(parse_opt_f64(""), None);
    }
}
