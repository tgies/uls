//! Update command - download and update the database.

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;

use uls_db::{Database, DatabaseConfig, ImportMode, Importer};
use uls_download::{DownloadConfig, DownloadProgress, DownloadResult, FccClient, ProgressCallback, ServiceCatalog};

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

pub async fn execute(service: &str, force: bool, minimal: bool) -> Result<()> {
    let db_path = default_db_path();
    let cache_path = default_cache_path();

    // Validate service
    let service_code = match service.to_lowercase().as_str() {
        "amateur" | "ham" => "HA",
        "gmrs" => "ZA",
        "all" => {
            eprintln!("'all' services not yet implemented. Use 'amateur' or 'gmrs'.");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Unknown service: {}. Use 'amateur' or 'gmrs'.", service);
            std::process::exit(1);
        }
    };

    let import_mode = if minimal { ImportMode::Minimal } else { ImportMode::Full };
    
    println!("Updating {} database (mode={:?})...", service, import_mode);
    println!("Database: {}", db_path.display());
    println!("Cache:    {}", cache_path.display());
    println!();

    // Initialize database if needed
    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;
    
    if !db.is_initialized()? {
        println!("Initializing database...");
        db.initialize()?;
    }

    // Set up download client
    let download_config = DownloadConfig::with_cache_dir(cache_path);
    let client = FccClient::new(download_config)?;

    // Get the complete license file
    let data_file = ServiceCatalog::complete_license(service_code)?;
    
    println!("Checking for updates: {}...", data_file.filename());
    
    // Create progress bar for download
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let progress_callback: ProgressCallback = Arc::new(move |progress: &DownloadProgress| {
        if let Some(total) = progress.total_bytes {
            pb.set_length(total);
            pb.set_position(progress.downloaded_bytes);
        }
    });

    let (zip_path, download_result) = client.download_file(&data_file, progress_callback).await?;
    
    // Read the cached ETag for this file
    let cache_etag = client.get_cached_etag(&data_file);
    
    // Get the ETag of what's currently imported in the DB for this service
    let db_imported_etag = db.get_imported_etag(service)?;
    
    // Service codes for this import
    let service_codes: Vec<&str> = match service_code {
        "HA" | "HV" => vec!["HA", "HV"],
        "ZA" => vec!["ZA"],
        _ => vec![service_code],
    };
    
    // Determine if we need to import
    let needs_import = match download_result {
        DownloadResult::Downloaded => {
            println!("\n✓ Downloaded new {} data.", service);
            true
        }
        DownloadResult::NotModified => {
            println!("\n✓ No new {} data available (FCC file unchanged).", service);
            if force {
                println!("  --force specified, re-importing from cache...");
                true
            } else {
                // Compare cache ETag with what's in the database
                match (&cache_etag, &db_imported_etag) {
                    (Some(cache), Some(db_etag)) if cache == db_etag => {
                        // Cache matches what's imported
                        let count = db.count_by_service(&service_codes)?;
                        println!("  Database already has this version ({} records).", count);
                        false
                    }
                    _ => {
                        // Cache differs from DB or no record - need to import
                        println!("  Database needs update. Importing from cache...");
                        true
                    }
                }
            }
        }
    };

    if !needs_import {
        return Ok(());
    }
    
    // Store the ETag we're about to import (will save at end after success)
    let import_etag = cache_etag.clone();

    println!();

    // Create progress bar for import
    let import_pb = ProgressBar::new_spinner();
    import_pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap());
    import_pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Use the library Importer
    println!("Importing records...");
    let importer = Importer::new(&db);
    
    let progress_cb = Box::new(move |p: &uls_db::ImportProgress| {
        import_pb.set_message(format!(
            "File {}/{}: {} ({} records)",
            p.current_file, p.total_files, p.file_name, p.records
        ));
    });
    
    let stats = importer.import_zip_with_mode(&zip_path, import_mode, Some(progress_cb))?;
    
    println!("  [{}] Imported {} records", 
        chrono::Utc::now().format("%H:%M:%S"),
        stats.records);

    // Set last updated
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    db.set_last_updated(&now)?;
    
    // Only save the imported ETag if there were no insert errors
    if stats.is_successful() {
        if let Some(etag) = import_etag {
            db.set_imported_etag(service, &etag)?;
        }
    }

    println!("✓ Imported {} records from {} files in {:.1}s ({:.0} records/sec)", 
        stats.records, stats.files, stats.duration_secs, stats.rate());
    
    // Report errors
    if stats.parse_errors > 0 || stats.insert_errors > 0 {
        println!();
        if stats.parse_errors > 0 {
            println!("⚠ {} parse errors (use -v to see details)", stats.parse_errors);
        }
        if stats.insert_errors > 0 {
            println!("⚠ {} insert errors (use -v to see details)", stats.insert_errors);
        }
    }
    
    println!("✓ Database updated: {}", now);

    Ok(())
}
