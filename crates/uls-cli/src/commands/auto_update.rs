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

