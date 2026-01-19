//! Update command - download and update the database with differential updates.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::{Datelike, NaiveDate, Utc};
use indicatif::{ProgressBar, ProgressStyle};

use uls_db::{Database, DatabaseConfig, ImportMode, Importer};
use uls_download::{DownloadConfig, DownloadProgress, FccClient, ProgressCallback, ServiceCatalog};
use uls_parser::archive::ZipExtractor;

use crate::config::{default_cache_path, default_db_path};

#[allow(dead_code)]
pub async fn execute(service: &str, force: bool, minimal: bool) -> Result<()> {
    execute_with_options(service, force, minimal, false, false).await
}

pub async fn execute_with_options(
    service: &str,
    force: bool,
    minimal: bool,
    daily_only: bool,
    check_only: bool,
) -> Result<()> {
    let db_path = default_db_path();
    let cache_path = default_cache_path();

    let service_code = match service.to_lowercase().as_str() {
        "amateur" | "ham" => "HA",
        "gmrs" => "ZA",
        "all" => bail!("'all' services not yet implemented"),
        _ => bail!("Unknown service: {}", service),
    };

    let import_mode = if minimal {
        ImportMode::Minimal
    } else {
        ImportMode::Full
    };

    let service_name = match service_code {
        "HA" => "amateur",
        "ZA" => "gmrs",
        _ => service_code,
    };

    println!("Updating {} database...", service_name);
    println!("Database: {}", db_path.display());

    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;

    if !db.is_initialized()? {
        println!("Initializing database...");
        db.initialize()?;
    } else {
        db.migrate_if_needed()?;
    }

    let download_config = DownloadConfig::with_cache_dir(cache_path);
    let client = FccClient::new(download_config)?;

    let result = run_update(
        &db,
        &client,
        service_code,
        &import_mode,
        force,
        daily_only,
        check_only,
    )
    .await?;

    match result {
        UpdateResult::UpToDate => println!("\n✓ Database is up to date."),
        UpdateResult::Updated { dailies, weekly } => {
            println!();
            if weekly {
                println!("✓ Applied weekly import.");
            }
            if dailies > 0 {
                println!("✓ Applied {} daily update(s).", dailies);
            }
        }
        UpdateResult::CheckOnly { available } => {
            println!("\n({} update(s) available, check mode)", available);
        }
    }

    Ok(())
}

enum UpdateResult {
    UpToDate,
    Updated { dailies: usize, weekly: bool },
    CheckOnly { available: usize },
}

async fn run_update(
    db: &Database,
    client: &FccClient,
    service_code: &str,
    import_mode: &ImportMode,
    force: bool,
    daily_only: bool,
    check_only: bool,
) -> Result<UpdateResult> {
    let today = Utc::now().date_naive();
    let db_weekly_date = db.get_last_weekly_date(service_code)?;
    let applied_patches: HashSet<NaiveDate> = db
        .get_applied_patches(service_code)?
        .into_iter()
        .map(|p| p.patch_date)
        .collect();

    // If no weekly in DB yet, we must import one
    if db_weekly_date.is_none() || force {
        if !daily_only {
            return apply_weekly_then_dailies(
                db,
                client,
                service_code,
                import_mode,
                today,
                check_only,
            )
            .await;
        }
    }

    let weekly_date = db_weekly_date.unwrap_or(today);

    // Try to build daily chain
    let chain_result =
        build_daily_chain(client, service_code, weekly_date, &applied_patches, today).await?;

    match chain_result {
        DailyChainResult::Complete(dailies) => {
            if dailies.is_empty() {
                return Ok(UpdateResult::UpToDate);
            }
            if check_only {
                return Ok(UpdateResult::CheckOnly {
                    available: dailies.len(),
                });
            }
            let count = apply_dailies(db, service_code, import_mode, &dailies)?;
            Ok(UpdateResult::Updated {
                dailies: count,
                weekly: false,
            })
        }
        DailyChainResult::Broken { missing_date } => {
            println!(
                "Daily chain broken at {}. Falling back to weekly import.",
                missing_date
            );
            if check_only {
                return Ok(UpdateResult::CheckOnly { available: 1 });
            }
            apply_weekly_then_dailies(db, client, service_code, import_mode, today, check_only)
                .await
        }
    }
}

enum DailyChainResult {
    Complete(Vec<(NaiveDate, PathBuf)>),
    Broken { missing_date: NaiveDate },
}

async fn build_daily_chain(
    client: &FccClient,
    service_code: &str,
    last_update: NaiveDate,
    applied: &HashSet<NaiveDate>,
    _today: NaiveDate,
) -> Result<DailyChainResult> {
    let full_name = ServiceCatalog::full_name(service_code).unwrap_or("amat");
    let weekdays = [
        uls_download::catalog::Weekday::Monday,
        uls_download::catalog::Weekday::Tuesday,
        uls_download::catalog::Weekday::Wednesday,
        uls_download::catalog::Weekday::Thursday,
        uls_download::catalog::Weekday::Friday,
        uls_download::catalog::Weekday::Saturday,
    ];

    let mut available_dailies: Vec<(NaiveDate, PathBuf)> = vec![];

    for weekday in &weekdays {
        let data_file = uls_download::DataFile::daily_license(full_name, *weekday);
        let progress: ProgressCallback = Arc::new(|_| {});

        match client.download_file(&data_file, progress).await {
            Ok((path, _)) => {
                if let Ok(Some(canonical_date)) = extract_canonical_date(&path) {
                    if canonical_date > last_update && !applied.contains(&canonical_date) {
                        available_dailies.push((canonical_date, path));
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if available_dailies.is_empty() {
        return Ok(DailyChainResult::Complete(vec![]));
    }

    // Sort by canonical date
    available_dailies.sort_by_key(|(date, _)| *date);

    // Check if we have a continuous chain from last_update
    let mut expected = last_update;
    for (date, _) in &available_dailies {
        // Allow gaps of 1 day (for Sundays)
        let gap = (*date - expected).num_days();
        if gap > 2 {
            return Ok(DailyChainResult::Broken {
                missing_date: expected.succ_opt().unwrap_or(expected),
            });
        }
        expected = *date;
    }

    Ok(DailyChainResult::Complete(available_dailies))
}

fn extract_canonical_date(zip_path: &PathBuf) -> Result<Option<NaiveDate>> {
    let mut extractor = ZipExtractor::open(zip_path)?;
    let date_str = match extractor.get_file_creation_date() {
        Some(s) => s,
        None => return Ok(None),
    };
    Ok(parse_fcc_date(&date_str))
}

fn parse_fcc_date(date_str: &str) -> Option<NaiveDate> {
    // Format: "Sun Jan 18 12:01:25 EST 2026"
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() >= 6 {
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
        let year: i32 = parts[5].parse().ok()?;
        NaiveDate::from_ymd_opt(year, month, day)
    } else {
        None
    }
}

async fn apply_weekly_then_dailies(
    db: &Database,
    client: &FccClient,
    service_code: &str,
    import_mode: &ImportMode,
    today: NaiveDate,
    check_only: bool,
) -> Result<UpdateResult> {
    if check_only {
        return Ok(UpdateResult::CheckOnly { available: 1 });
    }

    let weekly_date = apply_weekly(db, client, service_code, import_mode).await?;

    // Now try dailies after the fresh weekly
    let applied = HashSet::new();
    let chain = build_daily_chain(client, service_code, weekly_date, &applied, today).await?;

    let daily_count = if let DailyChainResult::Complete(dailies) = chain {
        apply_dailies(db, service_code, import_mode, &dailies)?
    } else {
        0
    };

    Ok(UpdateResult::Updated {
        dailies: daily_count,
        weekly: true,
    })
}

async fn apply_weekly(
    db: &Database,
    client: &FccClient,
    service_code: &str,
    import_mode: &ImportMode,
) -> Result<NaiveDate> {
    let data_file = ServiceCatalog::complete_license(service_code)?;

    println!("Downloading weekly file...");
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes}")?
            .progress_chars("#>-"),
    );

    let progress: ProgressCallback = Arc::new(move |p: &DownloadProgress| {
        if let Some(total) = p.total_bytes {
            pb.set_length(total);
            pb.set_position(p.downloaded_bytes);
        }
    });

    let (zip_path, _) = client.download_file(&data_file, progress).await?;
    let weekly_date = extract_canonical_date(&zip_path)?.unwrap_or_else(|| Utc::now().date_naive());

    println!("\nImporting weekly data...");

    // Count total records for progress bar
    let mut extractor = ZipExtractor::open(&zip_path)?;
    let counts = extractor.count_all_records()?;
    let total_records: usize = counts.values().sum();

    let import_pb = ProgressBar::new(total_records as u64);
    import_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} records")?
            .progress_chars("#>-"),
    );

    let import_progress: Option<Box<dyn Fn(&uls_db::ImportProgress) + Send + Sync>> =
        Some(Box::new(move |p| {
            import_pb.set_position(p.records as u64);
        }));

    let importer = Importer::new(db);
    let stats = importer.import_for_service(
        &zip_path,
        service_code,
        import_mode.clone(),
        import_progress,
    )?;

    println!(
        "\nImported {} records in {:.1}s",
        stats.records, stats.duration_secs
    );

    // Update metadata
    let etag = client.get_cached_etag(&data_file);
    if let Some(e) = etag {
        db.set_imported_etag(service_code, &e)?;
    }
    db.set_last_weekly_date(service_code, weekly_date)?;
    db.clear_applied_patches(service_code)?;

    if let Some(date_str) = ZipExtractor::open(&zip_path)?.get_file_creation_date() {
        db.set_last_updated(&date_str)?;
    }

    Ok(weekly_date)
}

fn apply_dailies(
    db: &Database,
    service_code: &str,
    import_mode: &ImportMode,
    dailies: &[(NaiveDate, PathBuf)],
) -> Result<usize> {
    let importer = Importer::new(db);
    let mut count = 0;

    for (date, path) in dailies {
        print!("  Applying {}... ", date);
        let stats = importer.import_patch(path, import_mode.clone(), None)?;
        println!("{} records", stats.records);

        // Update tracking
        if let Some(date_str) = ZipExtractor::open(path)?.get_file_creation_date() {
            db.set_last_updated(&date_str)?;
        }

        let weekday = match date.weekday() {
            chrono::Weekday::Mon => "mon",
            chrono::Weekday::Tue => "tue",
            chrono::Weekday::Wed => "wed",
            chrono::Weekday::Thu => "thu",
            chrono::Weekday::Fri => "fri",
            chrono::Weekday::Sat => "sat",
            chrono::Weekday::Sun => "sun",
        };

        db.record_applied_patch(service_code, *date, weekday, None, Some(stats.records))?;
        count += 1;
    }

    Ok(count)
}
