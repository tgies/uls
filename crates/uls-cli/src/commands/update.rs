//! Update command - download and update the database.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tracing::{info, warn, error};

use uls_db::{Database, DatabaseConfig};
use uls_download::{DownloadConfig, DownloadProgress, DownloadResult, FccClient, ProgressCallback, ServiceCatalog};
use uls_parser::archive::ZipExtractor;

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

pub async fn execute(service: &str, force: bool, _full_only: bool) -> Result<()> {
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

    println!("Updating {} database...", service);
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
    let cache_etag_path = zip_path.with_extension("zip.etag");
    let cache_etag = std::fs::read_to_string(&cache_etag_path).ok();
    
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

    // Import into database
    println!("Importing records...");
    let import_start = Instant::now();
    
    let mut extractor = ZipExtractor::open(&zip_path)
        .context("Failed to open ZIP file")?;

    let mut dat_files = extractor.list_dat_files();
    info!("Found {} DAT files in archive", dat_files.len());

    // Sort files to ensure HD.dat (licenses) is processed first
    // Other records have foreign key references to licenses
    dat_files.sort_by(|a, b| {
        let priority = |s: &str| -> u8 {
            let upper = s.to_uppercase();
            if upper.contains("HD") { 0 }      // Header/licenses first
            else if upper.contains("EN") { 1 } // Entities second
            else if upper.contains("AM") { 2 } // Amateur third
            else { 3 }                         // Everything else
        };
        priority(a).cmp(&priority(b))
    });
    
    info!("Processing order: {:?}", dat_files);

    // Create progress bar for import
    let import_pb = ProgressBar::new_spinner();
    import_pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap());
    import_pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut total_records = 0usize;
    let mut files_done = 0usize;
    let mut parse_errors = 0usize;
    let mut insert_errors = 0usize;

    // Optimize SQLite for bulk import
    {
        let conn = db.conn()?;
        conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;"  // 64MB cache
        )?;
    }

    // Begin transaction for bulk import
    let tx = db.begin_transaction()?;

    for dat_file in &dat_files {
        import_pb.set_message(format!(
            "File {}/{}: {} ({} records, {} errors)",
            files_done + 1,
            dat_files.len(),
            dat_file,
            total_records,
            parse_errors + insert_errors
        ));

        let mut file_records = 0usize;
        let mut file_parse_errors = 0usize;
        let mut file_insert_errors = 0usize;
        
        extractor.process_dat_streaming(dat_file, |line| {
            match line.to_record() {
                Ok(record) => {
                    if let Err(e) = tx.insert_record(&record) {
                        file_insert_errors += 1;
                        if file_insert_errors <= 5 {
                            warn!("Insert error in {}: {}", dat_file, e);
                        }
                    } else {
                        file_records += 1;
                    }
                }
                Err(e) => {
                    file_parse_errors += 1;
                    if file_parse_errors <= 5 {
                        warn!("Parse error in {}: {}", dat_file, e);
                    }
                }
            }
            
            // Update progress every 10k records
            if (file_records + file_parse_errors) % 10_000 == 0 {
                import_pb.set_message(format!(
                    "File {}/{}: {} ({} records)",
                    files_done + 1,
                    dat_files.len(),
                    dat_file,
                    total_records + file_records
                ));
            }
            true
        })?;
        
        total_records += file_records;
        parse_errors += file_parse_errors;
        insert_errors += file_insert_errors;
        files_done += 1;

        // Log file summary if there were errors
        if file_parse_errors > 0 || file_insert_errors > 0 {
            warn!(
                "{}: {} records, {} parse errors, {} insert errors",
                dat_file, file_records, file_parse_errors, file_insert_errors
            );
        }
    }

    import_pb.set_message("Committing transaction...");
    tx.commit()?;

    // Reset SQLite settings
    {
        let conn = db.conn()?;
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;"
        )?;
    }

    import_pb.finish_with_message(format!("Imported {} records", total_records));

    // Set last updated
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    db.set_last_updated(&now)?;
    
    // Only save the imported ETag if there were no insert errors
    // This ensures we'll re-import on next run if something went wrong
    if insert_errors == 0 {
        if let Some(etag) = import_etag {
            db.set_imported_etag(service, &etag)?;
        }
    }

    let elapsed = import_start.elapsed();
    let rate = total_records as f64 / elapsed.as_secs_f64();

    println!();
    println!("✓ Imported {} records from {} files in {:.1}s ({:.0} records/sec)", 
        total_records, dat_files.len(), elapsed.as_secs_f64(), rate);
    
    // Report errors
    if parse_errors > 0 || insert_errors > 0 {
        println!();
        if parse_errors > 0 {
            println!("⚠ {} parse errors (use -v to see details)", parse_errors);
        }
        if insert_errors > 0 {
            println!("⚠ {} insert errors (use -v to see details)", insert_errors);
        }
    }
    
    println!("✓ Database updated: {}", now);

    Ok(())
}
