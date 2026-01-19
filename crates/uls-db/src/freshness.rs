//! Data freshness detection and staleness utilities.
//!
//! Provides utilities for detecting stale data and determining what updates are needed.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Parse FCC datetime format: "Tue Jan 13 08:00:15 EST 2026"
fn parse_fcc_datetime(s: &str) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let month = match parts[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };

    let day: u32 = parts[2].parse().ok()?;
    let time_parts: Vec<&str> = parts[3].split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let min: u32 = time_parts[1].parse().ok()?;
    let sec: u32 = time_parts[2].parse().ok()?;
    let year: i32 = parts[5].parse().ok()?;

    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, min, sec)?;

    // EST is UTC-5, but we'll approximate as UTC for staleness purposes
    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

/// Default staleness threshold in days.
pub const DEFAULT_STALE_THRESHOLD_DAYS: i64 = 3;

/// Information about the freshness of data for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFreshness {
    /// Radio service code (e.g., "HA", "ZA").
    pub service: String,

    /// When the data was last updated.
    pub last_updated: Option<DateTime<Utc>>,

    /// Age of the data.
    pub age: Option<Duration>,

    /// Whether the data is considered stale.
    pub is_stale: bool,

    /// Age in human-readable format.
    pub age_display: String,

    /// Date of the last applied weekly import.
    pub last_weekly_date: Option<NaiveDate>,

    /// Dates of applied patches since last weekly.
    pub applied_patch_dates: Vec<NaiveDate>,

    /// Dates of missing patches (days that should have patches but don't).
    pub missing_patch_dates: Vec<NaiveDate>,
}

impl DataFreshness {
    /// Create a new DataFreshness with unknown/uninitialized state.
    pub fn unknown(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            last_updated: None,
            age: None,
            is_stale: true, // Unknown data is considered stale
            age_display: "unknown".to_string(),
            last_weekly_date: None,
            applied_patch_dates: Vec::new(),
            missing_patch_dates: Vec::new(),
        }
    }

    /// Create DataFreshness from a last_updated timestamp string.
    pub fn from_timestamp(
        service: impl Into<String>,
        timestamp: Option<&str>,
        threshold_days: i64,
    ) -> Self {
        let service = service.into();

        let last_updated = timestamp.and_then(|ts| {
            // Try parsing various formats
            DateTime::parse_from_rfc3339(ts)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    // Try "YYYY-MM-DD HH:MM:SS UTC" format
                    chrono::NaiveDateTime::parse_from_str(
                        ts.trim_end_matches(" UTC"),
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .ok()
                    .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
                })
                .or_else(|| {
                    // Try FCC format: "Tue Jan 13 08:00:15 EST 2026"
                    parse_fcc_datetime(ts)
                })
        });

        let now = Utc::now();
        let age = last_updated.map(|lu| now.signed_duration_since(lu));
        let threshold = Duration::days(threshold_days);
        let is_stale = age.map(|a| a > threshold).unwrap_or(true);

        let age_display = match age {
            Some(a) => {
                let days = a.num_days();
                let hours = a.num_hours() % 24;
                if days > 0 {
                    format!("{}d {}h", days, hours)
                } else if hours > 0 {
                    format!("{}h", hours)
                } else {
                    let mins = a.num_minutes();
                    format!("{}m", mins)
                }
            }
            None => "unknown".to_string(),
        };

        Self {
            service,
            last_updated,
            age,
            is_stale,
            age_display,
            last_weekly_date: None,
            applied_patch_dates: Vec::new(),
            missing_patch_dates: Vec::new(),
        }
    }

    /// Get the age in days, or None if unknown.
    pub fn age_days(&self) -> Option<i64> {
        self.age.map(|a| a.num_days())
    }

    /// Check if data needs a weekly update (new weekly available).
    pub fn needs_weekly_update(&self) -> bool {
        // If we don't know when last weekly was, assume we need one
        self.last_weekly_date.is_none()
    }

    /// Check if there are missing daily patches to apply.
    pub fn has_missing_patches(&self) -> bool {
        !self.missing_patch_dates.is_empty()
    }
}

/// Record of an applied daily patch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedPatch {
    /// Radio service code.
    pub service: String,

    /// Date of the patch (YYYY-MM-DD).
    pub patch_date: NaiveDate,

    /// Weekday abbreviation (mon, tue, etc.).
    pub weekday: String,

    /// When the patch was applied.
    pub applied_at: DateTime<Utc>,

    /// ETag of the downloaded file, if known.
    pub etag: Option<String>,

    /// Number of records in the patch.
    pub record_count: Option<usize>,
}

/// Staleness warning configuration.
#[derive(Debug, Clone)]
pub struct StalenessConfig {
    /// Threshold in days before data is considered stale.
    pub threshold_days: i64,

    /// Whether to show staleness warnings.
    pub warn_enabled: bool,

    /// Whether to auto-update when stale.
    pub auto_update: bool,
}

impl Default for StalenessConfig {
    fn default() -> Self {
        Self {
            threshold_days: DEFAULT_STALE_THRESHOLD_DAYS,
            warn_enabled: true,
            auto_update: false,
        }
    }
}

impl StalenessConfig {
    /// Create config with warnings disabled.
    pub fn no_warnings() -> Self {
        Self {
            warn_enabled: false,
            ..Default::default()
        }
    }

    /// Create config with auto-update enabled.
    pub fn with_auto_update() -> Self {
        Self {
            auto_update: true,
            ..Default::default()
        }
    }

    /// Create config with custom threshold.
    pub fn with_threshold(days: i64) -> Self {
        Self {
            threshold_days: days,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_freshness_from_recent_timestamp() {
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let freshness = DataFreshness::from_timestamp("HA", Some(&timestamp), 3);

        assert!(!freshness.is_stale);
        assert!(freshness.age.is_some());
        assert!(freshness.age_days().unwrap_or(999) < 1);
    }

    #[test]
    fn test_freshness_from_old_timestamp() {
        let old = Utc::now() - Duration::days(5);
        let timestamp = old.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let freshness = DataFreshness::from_timestamp("HA", Some(&timestamp), 3);

        assert!(freshness.is_stale);
        assert_eq!(freshness.age_days(), Some(5));
    }

    #[test]
    fn test_freshness_unknown() {
        let freshness = DataFreshness::from_timestamp("HA", None, 3);

        assert!(freshness.is_stale);
        assert!(freshness.last_updated.is_none());
        assert_eq!(freshness.age_display, "unknown");
    }

    #[test]
    fn test_staleness_config_defaults() {
        let config = StalenessConfig::default();

        assert_eq!(config.threshold_days, 3);
        assert!(config.warn_enabled);
        assert!(!config.auto_update);
    }

    #[test]
    fn test_age_display_formatting() {
        // Test hours display
        let recent = Utc::now() - Duration::hours(5);
        let timestamp = recent.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let freshness = DataFreshness::from_timestamp("HA", Some(&timestamp), 3);
        assert!(freshness.age_display.contains("h"));

        // Test days display
        let old = Utc::now() - Duration::days(2) - Duration::hours(3);
        let timestamp = old.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let freshness = DataFreshness::from_timestamp("HA", Some(&timestamp), 3);
        assert!(freshness.age_display.contains("d"));
    }

    #[test]
    fn test_parse_fcc_datetime() {
        let result = parse_fcc_datetime("Tue Jan 13 08:31:48 EST 2026");
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 13);
    }

    #[test]
    fn test_parse_fcc_datetime_invalid() {
        assert!(parse_fcc_datetime("invalid").is_none());
        assert!(parse_fcc_datetime("").is_none());
        assert!(parse_fcc_datetime("2026-01-13").is_none()); // Wrong format
    }

    #[test]
    fn test_freshness_from_fcc_format() {
        // FCC format: "Tue Jan 13 08:31:48 EST 2026"
        // This is older than 3 days from today (Jan 19), so should be stale
        let freshness =
            DataFreshness::from_timestamp("HA", Some("Tue Jan 13 08:31:48 EST 2026"), 3);
        assert!(freshness.is_stale);
        assert!(freshness.last_updated.is_some());
    }
}
