//! FCC ULS service and file catalog.
//!
//! Maps radio service codes to their corresponding FCC download files.

use crate::error::{DownloadError, Result};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A downloadable FCC ULS data file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataFile {
    /// The service abbreviation (e.g., "amat", "gmrs").
    pub service: String,

    /// The file type (license or application).
    pub file_type: FileType,

    /// The update type (complete or daily).
    pub update_type: UpdateType,

    /// For daily files, the day of week. None for complete files.
    pub day: Option<Weekday>,
}

impl DataFile {
    /// Create a new complete (weekly) license file.
    pub fn complete_license(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            file_type: FileType::License,
            update_type: UpdateType::Complete,
            day: None,
        }
    }

    /// Create a new complete (weekly) application file.
    pub fn complete_application(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            file_type: FileType::Application,
            update_type: UpdateType::Complete,
            day: None,
        }
    }

    /// Create a new daily license file.
    pub fn daily_license(service: impl Into<String>, day: Weekday) -> Self {
        Self {
            service: service.into(),
            file_type: FileType::License,
            update_type: UpdateType::Daily,
            day: Some(day),
        }
    }

    /// Get the filename for this data file.
    pub fn filename(&self) -> String {
        let prefix = match self.file_type {
            FileType::License => "l",
            FileType::Application => "a",
        };

        match self.update_type {
            UpdateType::Complete => format!("{}_{}.zip", prefix, self.service),
            UpdateType::Daily => {
                let day_abbrev = self.day.map(|d| d.abbrev()).unwrap_or("mon");
                // Daily files use abbreviated service names
                let daily_service = ServiceCatalog::daily_abbreviation(&self.service);
                format!("{}_{}_{}.zip", prefix, daily_service, day_abbrev)
            }
        }
    }

    /// Get the URL path for this data file (without base URL).
    pub fn url_path(&self) -> String {
        match self.update_type {
            UpdateType::Complete => format!("complete/{}", self.filename()),
            UpdateType::Daily => format!("daily/{}", self.filename()),
        }
    }
}

impl fmt::Display for DataFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.filename())
    }
}

/// Type of data file (license or application).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileType {
    /// License data (l_*.zip).
    License,
    /// Application data (a_*.zip).
    Application,
}

/// Type of update (complete weekly or daily incremental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateType {
    /// Complete weekly database.
    Complete,
    /// Daily transaction file.
    Daily,
}

/// Day of week for daily files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    /// Get all weekdays.
    pub const ALL: [Weekday; 6] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ];

    /// Get the three-letter abbreviation.
    pub fn abbrev(&self) -> &'static str {
        match self {
            Weekday::Monday => "mon",
            Weekday::Tuesday => "tue",
            Weekday::Wednesday => "wed",
            Weekday::Thursday => "thu",
            Weekday::Friday => "fri",
            Weekday::Saturday => "sat",
        }
    }

    /// Create from chrono::Weekday.
    pub fn from_chrono(day: chrono::Weekday) -> Option<Self> {
        match day {
            chrono::Weekday::Mon => Some(Weekday::Monday),
            chrono::Weekday::Tue => Some(Weekday::Tuesday),
            chrono::Weekday::Wed => Some(Weekday::Wednesday),
            chrono::Weekday::Thu => Some(Weekday::Thursday),
            chrono::Weekday::Fri => Some(Weekday::Friday),
            chrono::Weekday::Sat => Some(Weekday::Saturday),
            chrono::Weekday::Sun => None, // No Sunday files
        }
    }

    /// Get the weekday for a given date (if a daily file exists).
    pub fn for_date(date: NaiveDate) -> Option<Self> {
        Self::from_chrono(date.weekday())
    }
}

/// Catalog of FCC ULS services and their corresponding files.
pub struct ServiceCatalog;

impl ServiceCatalog {
    /// All supported services with their full and daily abbreviations.
    /// Format: (full_name, daily_abbreviation, description, radio_service_codes)
    const SERVICES: &'static [(
        &'static str,
        &'static str,
        &'static str,
        &'static [&'static str],
    )] = &[
        ("amat", "am", "Amateur Radio", &["HA", "HV"]),
        ("gmrs", "gm", "General Mobile Radio Service", &["ZA"]),
        ("ship", "sh", "Ship Stations", &["SA", "SB"]),
        ("coast", "co", "Coastal Stations", &["MC"]),
        ("aircraft", "ac", "Aircraft Stations", &["AC"]),
        ("market", "mk", "Market Based Services", &[]),
        ("land", "ln", "Land Mobile", &[]),
        ("micro", "mi", "Microwave", &[]),
        ("paging", "pg", "Paging", &[]),
    ];

    /// Get the daily abbreviation for a service.
    pub fn daily_abbreviation(service: &str) -> &'static str {
        Self::SERVICES
            .iter()
            .find(|(full, _, _, _)| *full == service)
            .map(|(_, abbrev, _, _)| *abbrev)
            .unwrap_or("xx") // Unknown services get placeholder
    }

    /// Get the full service name from an abbreviation or radio service code.
    /// Accepts: full name ("amat"), daily abbrev ("am"), or radio service code ("HA").
    pub fn full_name(input: &str) -> Option<&'static str> {
        Self::SERVICES
            .iter()
            .find(|(full, daily, _, codes)| {
                *full == input || *daily == input || codes.contains(&input)
            })
            .map(|(full, _, _, _)| *full)
    }

    /// Get all available services.
    pub fn all_services() -> Vec<ServiceInfo> {
        Self::SERVICES
            .iter()
            .map(|(name, abbrev, desc, codes)| ServiceInfo {
                name: name.to_string(),
                daily_abbrev: abbrev.to_string(),
                description: desc.to_string(),
                radio_service_codes: codes.iter().map(|s| s.to_string()).collect(),
            })
            .collect()
    }

    /// Check if a service is known.
    pub fn is_known_service(service: &str) -> bool {
        Self::SERVICES
            .iter()
            .any(|(full, daily, _, _)| *full == service || *daily == service)
    }

    /// Get complete license file for a service.
    pub fn complete_license(service: &str) -> Result<DataFile> {
        let full_name = Self::full_name(service)
            .ok_or_else(|| DownloadError::UnknownService(service.to_string()))?;
        Ok(DataFile::complete_license(full_name))
    }

    /// Get complete application file for a service.
    pub fn complete_application(service: &str) -> Result<DataFile> {
        let full_name = Self::full_name(service)
            .ok_or_else(|| DownloadError::UnknownService(service.to_string()))?;
        Ok(DataFile::complete_application(full_name))
    }

    /// Get all daily license files for a service.
    pub fn daily_licenses(service: &str) -> Result<Vec<DataFile>> {
        let full_name = Self::full_name(service)
            .ok_or_else(|| DownloadError::UnknownService(service.to_string()))?;

        Ok(Weekday::ALL
            .iter()
            .map(|day| DataFile::daily_license(full_name, *day))
            .collect())
    }

    /// Get the daily license file for a specific date.
    pub fn daily_license_for_date(service: &str, date: NaiveDate) -> Result<Option<DataFile>> {
        let full_name = Self::full_name(service)
            .ok_or_else(|| DownloadError::UnknownService(service.to_string()))?;

        Ok(Weekday::for_date(date).map(|day| DataFile::daily_license(full_name, day)))
    }
}

/// Information about a supported service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Full service name (e.g., "amat").
    pub name: String,
    /// Daily file abbreviation (e.g., "am").
    pub daily_abbrev: String,
    /// Human-readable description.
    pub description: String,
    /// Associated radio service codes.
    pub radio_service_codes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_license_filename() {
        let file = DataFile::complete_license("amat");
        assert_eq!(file.filename(), "l_amat.zip");
        assert_eq!(file.url_path(), "complete/l_amat.zip");
    }

    #[test]
    fn test_complete_application_filename() {
        let file = DataFile::complete_application("amat");
        assert_eq!(file.filename(), "a_amat.zip");
    }

    #[test]
    fn test_daily_license_filename() {
        let file = DataFile::daily_license("amat", Weekday::Monday);
        assert_eq!(file.filename(), "l_am_mon.zip");
        assert_eq!(file.url_path(), "daily/l_am_mon.zip");
    }

    #[test]
    fn test_gmrs_files() {
        let complete = DataFile::complete_license("gmrs");
        assert_eq!(complete.filename(), "l_gmrs.zip");

        let daily = DataFile::daily_license("gmrs", Weekday::Friday);
        assert_eq!(daily.filename(), "l_gm_fri.zip");
    }

    #[test]
    fn test_service_catalog() {
        assert!(ServiceCatalog::is_known_service("amat"));
        assert!(ServiceCatalog::is_known_service("am"));
        assert!(ServiceCatalog::is_known_service("gmrs"));
        assert!(!ServiceCatalog::is_known_service("unknown"));
    }

    #[test]
    fn test_daily_abbreviation() {
        assert_eq!(ServiceCatalog::daily_abbreviation("amat"), "am");
        assert_eq!(ServiceCatalog::daily_abbreviation("gmrs"), "gm");
    }

    #[test]
    fn test_all_services() {
        let services = ServiceCatalog::all_services();
        assert!(services.iter().any(|s| s.name == "amat"));
        assert!(services.iter().any(|s| s.name == "gmrs"));
    }

    #[test]
    fn test_radio_service_code_lookup() {
        // Radio service codes should map to full service names
        assert_eq!(ServiceCatalog::full_name("HA"), Some("amat"));
        assert_eq!(ServiceCatalog::full_name("HV"), Some("amat"));
        assert_eq!(ServiceCatalog::full_name("ZA"), Some("gmrs"));
    }

    #[test]
    fn test_complete_license_by_radio_service_code() {
        // CLI passes radio service codes like "HA" - this must work
        let file = ServiceCatalog::complete_license("HA").expect("HA should be recognized");
        assert_eq!(file.filename(), "l_amat.zip");

        let file = ServiceCatalog::complete_license("ZA").expect("ZA should be recognized");
        assert_eq!(file.filename(), "l_gmrs.zip");
    }

    #[test]
    fn test_complete_license_by_full_name() {
        let file = ServiceCatalog::complete_license("amat").expect("amat should be recognized");
        assert_eq!(file.filename(), "l_amat.zip");
    }

    #[test]
    fn test_unknown_service() {
        assert!(ServiceCatalog::complete_license("UNKNOWN").is_err());
    }
}
