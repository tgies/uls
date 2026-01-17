//! Auto-update helper for lazy loading.
//!
//! Checks if required data is available and triggers download/import if needed.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};
use uls_db::{Database, DatabaseConfig, ImportMode, Importer};
use uls_download::{DownloadConfig, DownloadProgress, FccClient, ProgressCallback as DownloadCallback, ServiceCatalog};

/// Get the default database path.
fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

/// Get the default cache path.
fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("uls")
}

/// Ensure the database has the required data for queries.
/// 
/// If data is missing, automatically downloads and imports it.
/// Returns the opened Database.
pub async fn ensure_data_available(service: &str) -> Result<Database> {
    let db_path = default_db_path();
    let cache_path = default_cache_path();

    // Open or create database
    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;
    
    // Initialize if needed
    if !db.is_initialized()? {
        db.initialize()?;
    }

    // Check if we have the basic data (HD + EN)
    let has_hd = db.has_record_type(service, "HD")?;
    let has_en = db.has_record_type(service, "EN")?;

    if has_hd && has_en {
        // Data is available
        return Ok(db);
    }

    // Need to download and import
    eprintln!("No data found for service '{}'. Downloading...", service);
    
    // Set up download client
    let download_config = DownloadConfig::with_cache_dir(cache_path);
    let client = FccClient::new(download_config)?;

    // Get the complete license file
    let data_file = ServiceCatalog::complete_license(service)?;
    
    // Create progress bar for download
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let progress_callback: DownloadCallback = Arc::new(move |progress: &DownloadProgress| {
        if let Some(total) = progress.total_bytes {
            pb.set_length(total);
            pb.set_position(progress.downloaded_bytes);
        }
    });

    let (zip_path, _) = client.download_file(&data_file, progress_callback).await?;
    
    eprintln!("Importing data (minimal mode for fast startup)...");
    
    // Import with minimal mode for fast startup
    let import_pb = ProgressBar::new_spinner();
    import_pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap());
    import_pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let importer = Importer::new(&db);
    
    let progress_cb = Box::new(move |p: &uls_db::ImportProgress| {
        import_pb.set_message(format!(
            "File {}/{}: {} ({} records)",
            p.current_file, p.total_files, p.file_name, p.records
        ));
    });
    
    let stats = importer.import_for_service(&zip_path, service, ImportMode::Minimal, Some(progress_cb))?;
    
    eprintln!("✓ Imported {} records from {} files", stats.records, stats.files);

    Ok(db)
}

/// Map user-friendly service name to service code.
pub fn service_name_to_code(service: &str) -> Option<&'static str> {
    match service.to_lowercase().as_str() {
        "amateur" | "ham" | "ha" => Some("HA"),
        "gmrs" | "za" => Some("ZA"),
        _ => None,
    }
}

/// Detect service type from callsign format.
/// 
/// **Amateur callsign formats** (letter-number-letter pattern):
/// - 1x2: K4AB (1 letter, 1 digit, 2 letters)
/// - 1x3: K4ABC (1 letter, 1 digit, 3 letters)
/// - 2x1: KB9A (2 letters, 1 digit, 1 letter)
/// - 2x2: KB9AB (2 letters, 1 digit, 2 letters)
/// - 2x3: KB9VBR (2 letters, 1 digit, 3 letters)
/// 
/// **GMRS callsign formats** (letters followed by digits, no embedded number):
/// - WQFX467, WRXX201 (3-4 letters + 3-4 digits)
/// - Legacy: KAE1234 format
pub fn detect_service_from_callsign(callsign: &str) -> &'static str {
    let upper = callsign.to_uppercase();
    let chars: Vec<char> = upper.chars().collect();
    
    if chars.is_empty() {
        return "HA"; // Default to amateur
    }
    
    // Amateur callsigns have a digit embedded in the middle, followed by letters
    // Pattern: [letters][digit][letters]
    // Find first digit position
    let first_digit_pos = chars.iter().position(|c| c.is_ascii_digit());
    
    if let Some(digit_pos) = first_digit_pos {
        // Check if there are letters AFTER the digit (amateur pattern)
        let after_digit: String = chars[digit_pos + 1..].iter().collect();
        if !after_digit.is_empty() && after_digit.chars().all(|c| c.is_ascii_alphabetic()) {
            // Has letters after digit = amateur callsign
            // Validate: 1-2 prefix letters, 1 digit, 1-3 suffix letters
            let prefix: String = chars[..digit_pos].iter().collect();
            if prefix.len() >= 1 && prefix.len() <= 2 
                && prefix.chars().all(|c| c.is_ascii_alphabetic())
                && after_digit.len() >= 1 && after_digit.len() <= 3
            {
                return "HA"; // Amateur
            }
        }
        
        // Check GMRS pattern: all letters, then all digits (at the end)
        let letters: String = chars.iter().take_while(|c| c.is_ascii_alphabetic()).collect();
        let digits: String = chars.iter().skip(letters.len()).collect();
        
        if letters.len() >= 3 && letters.len() <= 4
            && digits.len() >= 3 && digits.len() <= 4
            && digits.chars().all(|c| c.is_ascii_digit())
        {
            return "ZA"; // GMRS
        }
    }
    
    "HA" // Default to amateur
}


#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // service_name_to_code tests
    // ==========================================================================

    #[test]
    fn test_service_name_to_code_amateur() {
        assert_eq!(service_name_to_code("amateur"), Some("HA"));
        assert_eq!(service_name_to_code("Amateur"), Some("HA"));
        assert_eq!(service_name_to_code("AMATEUR"), Some("HA"));
        assert_eq!(service_name_to_code("ham"), Some("HA"));
        assert_eq!(service_name_to_code("ha"), Some("HA"));
    }

    #[test]
    fn test_service_name_to_code_gmrs() {
        assert_eq!(service_name_to_code("gmrs"), Some("ZA"));
        assert_eq!(service_name_to_code("GMRS"), Some("ZA"));
        assert_eq!(service_name_to_code("za"), Some("ZA"));
    }

    #[test]
    fn test_service_name_to_code_unknown() {
        assert_eq!(service_name_to_code("invalid"), None);
        assert_eq!(service_name_to_code("cb"), None);
        assert_eq!(service_name_to_code(""), None);
    }

    // ==========================================================================
    // detect_service_from_callsign tests - Amateur formats
    // ==========================================================================

    #[test]
    fn test_detect_amateur_1x2() {
        // 1x2: K0AA (1 letter, 1 digit, 2 letters)
        assert_eq!(detect_service_from_callsign("K0AA"), "HA");
        assert_eq!(detect_service_from_callsign("W1AW"), "HA");
        assert_eq!(detect_service_from_callsign("N5ZV"), "HA");
    }

    #[test]
    fn test_detect_amateur_1x3() {
        // 1x3: K0AAA (1 letter, 1 digit, 3 letters)
        assert_eq!(detect_service_from_callsign("K0AAA"), "HA");
        assert_eq!(detect_service_from_callsign("W1ABC"), "HA");
        assert_eq!(detect_service_from_callsign("N5WXY"), "HA");
    }

    #[test]
    fn test_detect_amateur_2x1() {
        // 2x1: AA0A (2 letters, 1 digit, 1 letter)
        assert_eq!(detect_service_from_callsign("AA0A"), "HA");
        assert_eq!(detect_service_from_callsign("KB9V"), "HA");
        assert_eq!(detect_service_from_callsign("WA5X"), "HA");
    }

    #[test]
    fn test_detect_amateur_2x2() {
        // 2x2: AA0AA (2 letters, 1 digit, 2 letters)
        assert_eq!(detect_service_from_callsign("AA0AA"), "HA");
        assert_eq!(detect_service_from_callsign("KB9AB"), "HA");
        assert_eq!(detect_service_from_callsign("WA5XY"), "HA");
    }

    #[test]
    fn test_detect_amateur_2x3() {
        // 2x3: AA0AAA (2 letters, 1 digit, 3 letters)
        assert_eq!(detect_service_from_callsign("KA0AAA"), "HA");
        assert_eq!(detect_service_from_callsign("KC5MQP"), "HA");
        assert_eq!(detect_service_from_callsign("WB9VBR"), "HA");
    }

    // ==========================================================================
    // detect_service_from_callsign tests - GMRS formats
    // ==========================================================================

    #[test]
    fn test_detect_gmrs_3x4() {
        // 3x4: KAA1234 (3 letters, 4 digits)
        assert_eq!(detect_service_from_callsign("KAA0219"), "ZA");
        assert_eq!(detect_service_from_callsign("KAB5678"), "ZA");
        assert_eq!(detect_service_from_callsign("KUE6893"), "ZA");
    }

    #[test]
    fn test_detect_gmrs_4x3() {
        // 4x3: WQFX467 (4 letters, 3 digits)
        assert_eq!(detect_service_from_callsign("WQFX467"), "ZA");
        assert_eq!(detect_service_from_callsign("WPTW543"), "ZA");
        assert_eq!(detect_service_from_callsign("WRAU623"), "ZA");
        assert_eq!(detect_service_from_callsign("WSAC374"), "ZA");
    }

    #[test]
    fn test_detect_callsign_case_insensitive() {
        assert_eq!(detect_service_from_callsign("w1aw"), "HA");
        assert_eq!(detect_service_from_callsign("wqfx467"), "ZA");
    }

    #[test]
    fn test_detect_callsign_empty() {
        assert_eq!(detect_service_from_callsign(""), "HA");
    }

    #[test]
    fn test_detect_callsign_real_data_amateur() {
        // Real callsigns from FCC database
        assert_eq!(detect_service_from_callsign("AJ8EO"), "HA");
        assert_eq!(detect_service_from_callsign("WK9CLS"), "HA");
        assert_eq!(detect_service_from_callsign("NI7D"), "HA");
    }

    #[test]
    fn test_detect_callsign_real_data_gmrs() {
        // Real GMRS callsigns from FCC database
        assert_eq!(detect_service_from_callsign("WQIY060"), "ZA");
        assert_eq!(detect_service_from_callsign("WSTZ988"), "ZA");
        assert_eq!(detect_service_from_callsign("KJL4040"), "ZA");
    }
}

