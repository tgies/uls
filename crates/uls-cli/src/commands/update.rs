//! Update command - download and update the database with differential updates.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::{Datelike, NaiveDate, Utc};
use indicatif::{ProgressBar, ProgressStyle};

use uls_db::{Database, DatabaseConfig, ImportMode, Importer};
use uls_download::{
    DownloadConfig, DownloadError, DownloadProgress, FccClient, ProgressCallback, ServiceCatalog,
};
use uls_parser::archive::ZipExtractor;

use crate::config::{default_cache_path, default_db_path};

/// Type alias for import progress callback to reduce type complexity.
type ImportProgressCallback = Option<Box<dyn Fn(&uls_db::ImportProgress) + Send + Sync>>;

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
        UpdateResult::Updated {
            dailies,
            weekly,
            gap,
        } => {
            println!();
            if weekly {
                println!("✓ Applied weekly import.");
            }
            if dailies > 0 {
                println!("✓ Applied {} daily update(s).", dailies);
            }
            if let Some(gap) = gap {
                print_daily_gap(gap);
            }
        }
        UpdateResult::Blocked { gap } => {
            println!("\nDatabase was not changed.");
            print_daily_gap(gap);
        }
        UpdateResult::CheckOnly { available, gap } => {
            println!("\n({} update(s) available, check mode)", available);
            if let Some(gap) = gap {
                print_daily_gap(gap);
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum UpdateResult {
    UpToDate,
    Updated {
        dailies: usize,
        weekly: bool,
        gap: Option<DailyGap>,
    },
    Blocked {
        gap: DailyGap,
    },
    CheckOnly {
        available: usize,
        gap: Option<DailyGap>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DailyGap {
    missing_date: NaiveDate,
    next_available_date: NaiveDate,
}

#[derive(Debug)]
struct DailyChain {
    contiguous: Vec<(NaiveDate, PathBuf)>,
    gap: Option<DailyGap>,
}

fn print_daily_gap(gap: DailyGap) {
    println!(
        "⚠ Daily updates are incomplete: {} is missing before the next available archive ({}).",
        gap.missing_date, gap.next_available_date
    );
    println!(
        "  Later daily archives were not applied; a missing daily or newer weekly is required."
    );
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
    if (db_weekly_date.is_none() || force) && !daily_only {
        return apply_weekly_then_dailies(
            db,
            client,
            service_code,
            import_mode,
            today,
            check_only,
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("unconditional weekly import was not applied"));
    }

    let weekly_date = db_weekly_date.unwrap_or(today);
    let coverage_date = applied_patches
        .iter()
        .copied()
        .filter(|date| *date > weekly_date)
        .max()
        .unwrap_or(weekly_date);

    // Try to build daily chain (no weekday to skip — we didn't just apply a weekly)
    let chain = build_daily_chain(
        client,
        service_code,
        weekly_date,
        &applied_patches,
        today,
        None,
    )
    .await?;

    if !chain.contiguous.is_empty() {
        if check_only {
            return Ok(UpdateResult::CheckOnly {
                available: chain.contiguous.len(),
                gap: chain.gap,
            });
        }
        // Persist independently useful prefix progress before attempting weekly
        // recovery. If a gap remains, the next invocation evaluates the weekly
        // path against the newly advanced coverage date.
        let count = apply_dailies(db, service_code, import_mode, &chain.contiguous)?;
        return Ok(UpdateResult::Updated {
            dailies: count,
            weekly: false,
            gap: chain.gap,
        });
    }

    let Some(gap) = chain.gap else {
        return Ok(UpdateResult::UpToDate);
    };

    if daily_only {
        return Ok(if check_only {
            UpdateResult::CheckOnly {
                available: 0,
                gap: Some(gap),
            }
        } else {
            UpdateResult::Blocked { gap }
        });
    }

    println!(
        "Daily chain is blocked at {}. Checking for a newer weekly import.",
        gap.missing_date
    );
    if check_only {
        return Ok(UpdateResult::CheckOnly {
            available: 0,
            gap: Some(gap),
        });
    }

    match apply_weekly_then_dailies(
        db,
        client,
        service_code,
        import_mode,
        today,
        false,
        Some(coverage_date),
    )
    .await?
    {
        Some(result) => Ok(result),
        None => Ok(UpdateResult::Blocked { gap }),
    }
}

/// Return the weekdays to check for daily files, optionally skipping the
/// weekday that coincides with a just-applied weekly snapshot.
fn weekdays_to_check(
    skip_weekday: Option<uls_download::catalog::Weekday>,
) -> Vec<uls_download::catalog::Weekday> {
    uls_download::catalog::Weekday::ALL
        .iter()
        .copied()
        .filter(|w| {
            if skip_weekday == Some(*w) {
                tracing::debug!("Skipping {} daily (same day as weekly)", w.abbrev());
                false
            } else {
                true
            }
        })
        .collect()
}

fn weekly_covered_daily_weekday(weekly_date: NaiveDate) -> uls_download::catalog::Weekday {
    let covered_data_date = weekly_date.pred_opt().unwrap_or(weekly_date);
    uls_download::catalog::Weekday::for_date(covered_data_date)
}

async fn build_daily_chain(
    client: &FccClient,
    service_code: &str,
    last_update: NaiveDate,
    applied: &HashSet<NaiveDate>,
    today: NaiveDate,
    skip_weekday: Option<uls_download::catalog::Weekday>,
) -> Result<DailyChain> {
    let full_name = ServiceCatalog::full_name(service_code).unwrap_or("amat");
    let chain_anchor = match contiguous_patch_coverage(last_update, applied) {
        Ok(date) => date,
        Err(gap) => {
            return Ok(DailyChain {
                contiguous: vec![],
                gap: Some(gap),
            });
        }
    };

    let mut available_dailies: Vec<(NaiveDate, PathBuf)> = vec![];
    let weekdays = weekdays_to_check(skip_weekday);

    for weekday in &weekdays {
        let data_file = uls_download::DataFile::daily_license(full_name, *weekday);
        let progress: ProgressCallback = Arc::new(|_| {});

        match client.download_file(&data_file, progress).await {
            Ok((path, _)) => {
                let canonical_date = extract_canonical_date(&path)?.ok_or_else(|| {
                    anyhow::anyhow!(
                        "daily archive {} has no valid FCC creation date",
                        path.display()
                    )
                })?;
                if canonical_date > today {
                    bail!(
                        "daily archive {} has future FCC creation date {} (today is {})",
                        path.display(),
                        canonical_date,
                        today
                    );
                }
                if canonical_date > chain_anchor {
                    available_dailies.push((canonical_date, path));
                }
            }
            Err(DownloadError::NotFound { .. }) => tracing::debug!(
                weekday = %weekday.abbrev(),
                "daily archive is not published"
            ),
            Err(error) => return Err(error.into()),
        }
    }

    if available_dailies.is_empty() {
        return Ok(DailyChain {
            contiguous: vec![],
            gap: None,
        });
    }

    // Sort by canonical date
    available_dailies.sort_by_key(|(date, _)| *date);

    let mut contiguous = Vec::new();
    let mut expected = chain_anchor;
    for (date, path) in available_dailies {
        if date <= expected {
            continue;
        }
        let next_expected = expected.succ_opt().unwrap_or(expected);
        if date > next_expected {
            return Ok(DailyChain {
                contiguous,
                gap: Some(DailyGap {
                    missing_date: next_expected,
                    next_available_date: date,
                }),
            });
        }
        expected = date;
        contiguous.push((date, path));
    }

    Ok(DailyChain {
        contiguous,
        gap: None,
    })
}

fn contiguous_patch_coverage(
    weekly_date: NaiveDate,
    applied: &HashSet<NaiveDate>,
) -> std::result::Result<NaiveDate, DailyGap> {
    let mut dates: Vec<_> = applied
        .iter()
        .copied()
        .filter(|date| *date > weekly_date)
        .collect();
    dates.sort_unstable();

    let mut coverage_date = weekly_date;
    for date in dates {
        let expected = coverage_date.succ_opt().unwrap_or(coverage_date);
        if date > expected {
            return Err(DailyGap {
                missing_date: expected,
                next_available_date: date,
            });
        }
        coverage_date = date;
    }

    Ok(coverage_date)
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
    newer_than: Option<NaiveDate>,
) -> Result<Option<UpdateResult>> {
    if check_only {
        return Ok(Some(UpdateResult::CheckOnly {
            available: 1,
            gap: None,
        }));
    }

    let Some(weekly_date) =
        apply_weekly(db, client, service_code, import_mode, newer_than, today).await?
    else {
        return Ok(None);
    };

    // A weekly created on Sunday D includes data through Saturday and therefore
    // subsumes the Saturday daily created on D. The Sunday daily is created on
    // D+1 and remains the first required archive after the weekly.
    let applied = HashSet::new();
    let skip = Some(weekly_covered_daily_weekday(weekly_date));
    let chain = build_daily_chain(client, service_code, weekly_date, &applied, today, skip).await?;

    let daily_count = apply_dailies(db, service_code, import_mode, &chain.contiguous)?;

    Ok(Some(UpdateResult::Updated {
        dailies: daily_count,
        weekly: true,
        gap: chain.gap,
    }))
}

async fn apply_weekly(
    db: &Database,
    client: &FccClient,
    service_code: &str,
    import_mode: &ImportMode,
    newer_than: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<Option<NaiveDate>> {
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
    let weekly_date = extract_canonical_date(&zip_path)?.ok_or_else(|| {
        anyhow::anyhow!(
            "weekly archive {} has no valid FCC creation date",
            zip_path.display()
        )
    })?;
    if weekly_date > today {
        bail!(
            "weekly archive {} has future FCC creation date {} (today is {})",
            zip_path.display(),
            weekly_date,
            today
        );
    }
    if let Some(coverage_date) = newer_than {
        if weekly_date <= coverage_date {
            println!(
                "\nWeekly archive {} does not advance current coverage through {}.",
                weekly_date, coverage_date
            );
            return Ok(None);
        }
    }

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

    let import_progress: ImportProgressCallback = Some(Box::new(move |p| {
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

    Ok(Some(weekly_date))
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

#[cfg(test)]
mod tests {
    use super::*;
    use uls_download::catalog::Weekday;

    #[test]
    fn test_parse_fcc_date_standard() {
        let date = parse_fcc_date("Sun Jan 18 12:01:25 EST 2026");
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 1, 18));
    }

    #[test]
    fn test_parse_fcc_date_various_months() {
        assert_eq!(
            parse_fcc_date("Mon Mar 03 08:00:00 EST 2025"),
            NaiveDate::from_ymd_opt(2025, 3, 3)
        );
        assert_eq!(
            parse_fcc_date("Fri Dec 31 23:59:59 EST 2027"),
            NaiveDate::from_ymd_opt(2027, 12, 31)
        );
    }

    #[test]
    fn test_parse_fcc_date_invalid() {
        assert!(parse_fcc_date("not a date").is_none());
        assert!(parse_fcc_date("").is_none());
        assert!(parse_fcc_date("Mon Xyz 01 00:00:00 EST 2025").is_none());
    }

    #[test]
    fn test_weekly_skips_daily_already_covered_by_snapshot() {
        // A weekly created on Sunday includes Saturday's data, so the Saturday
        // daily has the same canonical date and is the redundant archive.
        let sunday = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        assert_eq!(weekly_covered_daily_weekday(sunday), Weekday::Saturday);
    }

    #[test]
    fn test_weekdays_to_check_skips_sunday() {
        let kept = weekdays_to_check(Some(Weekday::Sunday));
        assert_eq!(kept.len(), 6);
        assert!(!kept.contains(&Weekday::Sunday));
    }

    #[test]
    fn test_weekdays_to_check_skips_correct_day() {
        for skip_day in &Weekday::ALL {
            let kept = weekdays_to_check(Some(*skip_day));
            assert_eq!(kept.len(), 6);
            assert!(!kept.contains(skip_day));
        }
    }

    #[test]
    fn test_weekdays_to_check_none_keeps_all() {
        let kept = weekdays_to_check(None);
        assert_eq!(kept.len(), 7);
    }

    #[test]
    fn test_contiguous_patch_coverage_accepts_complete_sequence() {
        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        let applied = HashSet::from([
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        ]);

        assert_eq!(
            contiguous_patch_coverage(weekly, &applied),
            Ok(NaiveDate::from_ymd_opt(2026, 1, 20).unwrap())
        );
    }

    #[test]
    fn test_contiguous_patch_coverage_rejects_metadata_gap() {
        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        let applied = HashSet::from([NaiveDate::from_ymd_opt(2026, 1, 20).unwrap()]);

        assert_eq!(
            contiguous_patch_coverage(weekly, &applied),
            Err(DailyGap {
                missing_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
                next_available_date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            })
        );
    }

    // =========================================================================
    // Orchestration tests (run_update / apply_weekly / build_daily_chain /
    // apply_dailies) driven by a wiremock backend and a real TempDir database.
    // =========================================================================

    use std::fs;
    use std::io::Write as _;
    use std::path::Path;
    use tempfile::TempDir;
    use uls_download::DownloadConfig;
    use wiremock::matchers::{method, path as wm_path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Path to the shared FCC sample fixtures.
    fn fixture_dir(service: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/fcc-sample")
            .join(service)
    }

    /// Build a ULS ZIP from the fixture DAT files plus a `counts` file carrying
    /// the given FCC creation date string. The creation date is what
    /// `extract_canonical_date` reads to order/gate weekly and daily files.
    fn build_fixture_zip(service: &str, creation_date: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("counts", opts).unwrap();
            writeln!(zip, "File Creation Date: {}", creation_date).unwrap();

            for entry in fs::read_dir(fixture_dir(service)).unwrap() {
                let p = entry.unwrap().path();
                if p.extension().is_some_and(|e| e == "dat") {
                    let name = p.file_name().unwrap().to_str().unwrap().to_string();
                    let contents = fs::read(&p).unwrap();
                    zip.start_file(name, opts).unwrap();
                    zip.write_all(&contents).unwrap();
                }
            }
            zip.finish().unwrap();
        }
        buf
    }

    fn build_counts_only_zip(creation_date: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("counts", opts).unwrap();
            writeln!(zip, "File Creation Date: {}", creation_date).unwrap();
            writeln!(zip, "     0 total").unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    /// Mount a ZIP body at the given URL path on the mock server.
    async fn mount_zip(server: &MockServer, url_path: &str, body: Vec<u8>) {
        Mock::given(method("GET"))
            .and(wm_path(url_path.to_string()))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(body.clone())
                    .insert_header("Content-Length", body.len().to_string())
                    .insert_header("ETag", "\"fixture-etag\""),
            )
            .mount(server)
            .await;
    }

    /// Open a fresh initialized database in a temp directory.
    fn fresh_db(dir: &Path) -> Database {
        let config = DatabaseConfig::with_path(dir.join("test.db"));
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();
        db
    }

    fn test_client(server: &MockServer, cache: &Path) -> FccClient {
        let mut config = DownloadConfig::with_cache_dir(cache.to_path_buf())
            .with_base_url(server.uri())
            .with_timeout(std::time::Duration::from_secs(10));
        config.max_retries = 0;
        FccClient::new(config).unwrap()
    }

    #[tokio::test]
    async fn test_build_daily_chain_rejects_future_archive_date() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let client = test_client(&server, tmp.path());

        mount_zip(
            &server,
            "/daily/l_am_mon.zip",
            build_fixture_zip("l_amat", "Tue Jan 20 08:00:00 EST 2026"),
        )
        .await;

        let error = build_daily_chain(
            &client,
            "HA",
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            &HashSet::new(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            None,
        )
        .await
        .unwrap_err();

        assert!(
            error.to_string().contains("future FCC creation date"),
            "unexpected error: {error:#}"
        );
    }

    #[tokio::test]
    async fn test_build_daily_chain_propagates_server_errors() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let client = test_client(&server, tmp.path());

        Mock::given(method("GET"))
            .and(wm_path("/daily/l_am_sun.zip"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let error = build_daily_chain(
            &client,
            "HA",
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            &HashSet::new(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            None,
        )
        .await
        .unwrap_err();

        assert!(
            error.to_string().contains("server error 503"),
            "unexpected error: {error:#}"
        );
    }

    #[tokio::test]
    async fn test_apply_weekly_rejects_future_archive_date() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Tue Jan 20 08:00:00 EST 2026"),
        )
        .await;

        let error = apply_weekly(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            None,
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        )
        .await
        .unwrap_err();

        assert!(
            error.to_string().contains("future FCC creation date"),
            "unexpected error: {error:#}"
        );
        assert!(db.get_last_weekly_date("HA").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_run_update_fresh_db_applies_weekly_and_dailies() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // Weekly published Sunday 2026-01-18 includes data through Saturday.
        // FCC stamps each daily the next morning, so Sunday data is canonical
        // Monday 2026-01-19 and Monday data is canonical Tuesday 2026-01-20.
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jan 18 12:01:25 EST 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_sun.zip",
            build_counts_only_zip("Mon Jan 19 08:00:00 EST 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_mon.zip",
            build_fixture_zip("l_amat", "Tue Jan 20 08:00:00 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Updated {
                dailies, weekly, ..
            } => {
                assert!(weekly, "weekly should be applied on a fresh DB");
                assert_eq!(dailies, 2, "Sunday and Monday dailies should apply");
            }
            other => panic!("expected Updated, got {:?}", DebugResult(&other)),
        }

        // Metadata reflects the weekly snapshot and both applied patches.
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18)
        );
        let patches = db.get_applied_patches("HA").unwrap();
        assert_eq!(patches.len(), 2);
        assert_eq!(
            patches
                .iter()
                .find(|patch| { patch.patch_date == NaiveDate::from_ymd_opt(2026, 1, 19).unwrap() })
                .and_then(|patch| patch.record_count),
            Some(0)
        );
        assert_eq!(
            patches
                .iter()
                .map(|patch| patch.patch_date)
                .collect::<HashSet<_>>(),
            HashSet::from([
                NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            ])
        );

        let retry = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();
        assert!(matches!(retry, UpdateResult::UpToDate));
    }

    #[tokio::test]
    async fn test_run_update_fresh_weekly_applies_full_daily_week() {
        // Regression: after a fresh weekly the chain starts with the Sunday
        // daily created on D+1, then continues through the weekday dailies.
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jan 18 12:01:25 EST 2026"),
        )
        .await;
        // Each daily is stamped the next morning (data day + 1).
        for (day, stamp) in [
            ("sun", "Mon Jan 19 08:00:00 EST 2026"),
            ("mon", "Tue Jan 20 08:00:00 EST 2026"),
            ("tue", "Wed Jan 21 08:00:00 EST 2026"),
            ("wed", "Thu Jan 22 08:00:00 EST 2026"),
            ("thu", "Fri Jan 23 08:00:00 EST 2026"),
            ("fri", "Sat Jan 24 08:00:00 EST 2026"),
        ] {
            mount_zip(
                &server,
                &format!("/daily/l_am_{day}.zip"),
                build_fixture_zip("l_amat", stamp),
            )
            .await;
        }

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Updated {
                dailies, weekly, ..
            } => {
                assert!(weekly, "weekly should be applied on a fresh DB");
                assert_eq!(dailies, 6, "all six post-weekly dailies should apply");
            }
            other => panic!("expected Updated, got {:?}", DebugResult(&other)),
        }

        let mut dates: Vec<_> = db
            .get_applied_patches("HA")
            .unwrap()
            .iter()
            .map(|p| p.patch_date)
            .collect();
        dates.sort();
        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 21).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 22).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 23).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 24).unwrap(),
            ]
        );
    }

    #[tokio::test]
    async fn test_run_update_up_to_date_returns_uptodate() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // Record an existing weekly far in the future so no daily is newer.
        let weekly = NaiveDate::from_ymd_opt(2030, 6, 7).unwrap(); // a Friday
        db.set_last_weekly_date("HA", weekly).unwrap();

        // Serve dailies whose canonical dates predate the weekly, so the chain
        // builder gates them all out.
        mount_zip(
            &server,
            "/daily/l_am_sun.zip",
            build_fixture_zip("l_amat", "Mon Jan 19 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        assert!(matches!(result, UpdateResult::UpToDate));
    }

    #[tokio::test]
    async fn test_run_update_check_only_reports_available_count() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // Existing weekly on Sunday; the Sunday daily created Monday is
        // available.
        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        mount_zip(
            &server,
            "/daily/l_am_sun.zip",
            build_fixture_zip("l_amat", "Mon Jan 19 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            true, // check_only
        )
        .await
        .unwrap();

        match result {
            UpdateResult::CheckOnly { available, .. } => assert_eq!(available, 1),
            other => panic!("expected CheckOnly, got {:?}", DebugResult(&other)),
        }

        // Check mode must not mutate patch state.
        assert!(db.get_applied_patches("HA").unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_run_update_check_only_fresh_db_reports_weekly() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // No weekly recorded and check_only: a weekly import is "available".
        let result = run_update(&db, &client, "HA", &ImportMode::Minimal, false, false, true)
            .await
            .unwrap();

        match result {
            UpdateResult::CheckOnly { available, .. } => assert_eq!(available, 1),
            other => panic!("expected CheckOnly, got {:?}", DebugResult(&other)),
        }
        // Nothing imported.
        assert!(db.get_last_weekly_date("HA").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_run_update_daily_only_no_newer_dailies_is_uptodate() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // daily_only with no weekly in DB: weekly_date defaults to today, and no
        // served daily is newer than today, so the chain is empty.
        mount_zip(
            &server,
            "/daily/l_am_sun.zip",
            build_fixture_zip("l_amat", "Mon Jan 19 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            true, // daily_only
            false,
        )
        .await
        .unwrap();

        assert!(matches!(result, UpdateResult::UpToDate));
        // daily_only must not perform a weekly import.
        assert!(db.get_last_weekly_date("HA").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_run_update_applies_contiguous_prefix_before_later_gap() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 7, 12).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        for date in 13..=16 {
            db.record_applied_patch(
                "HA",
                NaiveDate::from_ymd_opt(2026, 7, date).unwrap(),
                "fixture",
                None,
                Some(1),
            )
            .unwrap();
        }

        mount_zip(
            &server,
            "/daily/l_am_thu.zip",
            build_fixture_zip("l_amat", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jul 23 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jul 19 12:01:25 EDT 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Updated {
                dailies,
                weekly,
                gap: Some(gap),
            } => {
                assert_eq!(dailies, 1);
                assert!(!weekly);
                assert_eq!(
                    gap.missing_date,
                    NaiveDate::from_ymd_opt(2026, 7, 18).unwrap()
                );
                assert_eq!(
                    gap.next_available_date,
                    NaiveDate::from_ymd_opt(2026, 7, 23).unwrap()
                );
            }
            other => panic!("expected gapped Updated, got {:?}", DebugResult(&other)),
        }

        let dates: HashSet<_> = db
            .get_applied_patches("HA")
            .unwrap()
            .into_iter()
            .map(|patch| patch.patch_date)
            .collect();
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2026, 7, 17).unwrap()));
        assert!(!dates.contains(&NaiveDate::from_ymd_opt(2026, 7, 23).unwrap()));
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(weekly));
    }

    #[tokio::test]
    async fn test_run_update_retry_preserves_prefix_when_weekly_is_stale() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 7, 12).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        for date in 13..=20 {
            db.record_applied_patch(
                "HA",
                NaiveDate::from_ymd_opt(2026, 7, date).unwrap(),
                "fixture",
                None,
                Some(1),
            )
            .unwrap();
        }

        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jul 23 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jul 19 12:01:25 EDT 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Blocked { gap } => {
                assert_eq!(
                    gap.missing_date,
                    NaiveDate::from_ymd_opt(2026, 7, 21).unwrap()
                );
                assert_eq!(
                    gap.next_available_date,
                    NaiveDate::from_ymd_opt(2026, 7, 23).unwrap()
                );
            }
            other => panic!("expected Blocked, got {:?}", DebugResult(&other)),
        }
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(weekly));
        assert_eq!(db.get_applied_patches("HA").unwrap().len(), 8);
    }

    #[tokio::test]
    async fn test_run_update_invalid_weekly_date_preserves_prefix() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 7, 12).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        for date in 13..=17 {
            db.record_applied_patch(
                "HA",
                NaiveDate::from_ymd_opt(2026, 7, date).unwrap(),
                "fixture",
                None,
                Some(1),
            )
            .unwrap();
        }

        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jul 23 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "not a valid FCC date"),
        )
        .await;

        let error = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap_err();

        assert!(
            error.to_string().contains("no valid FCC creation date"),
            "unexpected error: {error:#}"
        );
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(weekly));
        assert_eq!(db.get_applied_patches("HA").unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_run_update_fresh_weekly_applies_prefix_before_later_gap() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jan 18 12:01:25 EST 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_sun.zip",
            build_fixture_zip("l_amat", "Mon Jan 19 08:00:00 EST 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_mon.zip",
            build_fixture_zip("l_amat", "Tue Jan 20 08:00:00 EST 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_thu.zip",
            build_fixture_zip("l_amat", "Fri Jan 23 08:00:00 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Updated {
                dailies,
                weekly,
                gap: Some(gap),
            } => {
                assert!(weekly);
                assert_eq!(dailies, 2);
                assert_eq!(
                    gap.missing_date,
                    NaiveDate::from_ymd_opt(2026, 1, 21).unwrap()
                );
                assert_eq!(
                    gap.next_available_date,
                    NaiveDate::from_ymd_opt(2026, 1, 23).unwrap()
                );
            }
            other => panic!("expected gapped Updated, got {:?}", DebugResult(&other)),
        }
        let patches = db.get_applied_patches("HA").unwrap();
        assert_eq!(patches.len(), 2);
        assert_eq!(
            patches
                .iter()
                .map(|patch| patch.patch_date)
                .collect::<HashSet<_>>(),
            HashSet::from([
                NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            ])
        );
    }

    #[tokio::test]
    async fn test_run_update_daily_only_never_falls_back_across_gap() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jan 22 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(&db, &client, "HA", &ImportMode::Minimal, false, true, false)
            .await
            .unwrap();

        assert!(matches!(result, UpdateResult::Blocked { .. }));
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(weekly));
        assert!(db.get_applied_patches("HA").unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_run_update_check_only_reports_safe_prefix_and_gap() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 7, 12).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        for date in 13..=16 {
            db.record_applied_patch(
                "HA",
                NaiveDate::from_ymd_opt(2026, 7, date).unwrap(),
                "fixture",
                None,
                Some(1),
            )
            .unwrap();
        }
        mount_zip(
            &server,
            "/daily/l_am_thu.zip",
            build_fixture_zip("l_amat", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jul 23 08:00:00 EDT 2026"),
        )
        .await;

        let result = run_update(&db, &client, "HA", &ImportMode::Minimal, false, false, true)
            .await
            .unwrap();

        match result {
            UpdateResult::CheckOnly {
                available: 1,
                gap: Some(gap),
            } => {
                assert_eq!(
                    gap.missing_date,
                    NaiveDate::from_ymd_opt(2026, 7, 18).unwrap()
                );
            }
            other => panic!("expected gapped CheckOnly, got {:?}", DebugResult(&other)),
        }
        assert_eq!(db.get_applied_patches("HA").unwrap().len(), 4);
    }

    #[tokio::test]
    async fn test_run_update_broken_daily_chain_falls_back_to_weekly() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        // Existing weekly on Sunday 2026-01-18. The only available daily is for
        // Thursday 2026-01-22, leaving a multi-day gap (Mon/Tue/Wed missing),
        // which the chain builder reports as broken and falls back to weekly.
        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();

        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jan 22 12:01:25 EST 2026"),
        )
        .await;
        // Fresh weekly served for the fallback import.
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Sun Jan 25 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            false,
        )
        .await
        .unwrap();

        match result {
            UpdateResult::Updated { weekly, .. } => {
                assert!(weekly, "fallback should perform a weekly import");
            }
            other => panic!("expected Updated, got {:?}", DebugResult(&other)),
        }

        // The fresh weekly date replaced the old one, and patches were cleared.
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 25)
        );
    }

    #[tokio::test]
    async fn test_run_update_broken_chain_check_only_reports_gap_without_update() {
        let server = MockServer::start().await;
        let tmp = TempDir::new().unwrap();
        let db = fresh_db(tmp.path());
        let client = test_client(&server, tmp.path());

        let weekly = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
        db.set_last_weekly_date("HA", weekly).unwrap();
        mount_zip(
            &server,
            "/daily/l_am_wed.zip",
            build_fixture_zip("l_amat", "Thu Jan 22 12:01:25 EST 2026"),
        )
        .await;

        let result = run_update(
            &db,
            &client,
            "HA",
            &ImportMode::Minimal,
            false,
            false,
            true, // check_only
        )
        .await
        .unwrap();

        match result {
            UpdateResult::CheckOnly {
                available,
                gap: Some(gap),
            } => {
                assert_eq!(available, 0);
                assert_eq!(
                    gap.missing_date,
                    NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()
                );
            }
            other => panic!("expected CheckOnly, got {:?}", DebugResult(&other)),
        }
        // Old weekly untouched in check mode.
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(weekly));
    }

    #[tokio::test]
    async fn test_extract_canonical_date_round_trip() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("l_amat.zip");
        let body = build_fixture_zip("l_amat", "Sun Jan 18 12:01:25 EST 2026");
        fs::write(&zip_path, body).unwrap();

        let date = extract_canonical_date(&zip_path).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 1, 18));
    }

    #[tokio::test]
    async fn test_extract_canonical_date_missing_counts_is_none() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("no_counts.zip");
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("HD.dat", opts).unwrap();
            zip.write_all(b"HD|1|||TEST|A|HA|\n").unwrap();
            zip.finish().unwrap();
        }
        fs::write(&zip_path, buf).unwrap();

        assert_eq!(extract_canonical_date(&zip_path).unwrap(), None);
    }

    /// Wrapper that makes the private `UpdateResult` printable in panic messages.
    struct DebugResult<'a>(&'a UpdateResult);
    impl std::fmt::Debug for DebugResult<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.0 {
                UpdateResult::UpToDate => write!(f, "UpToDate"),
                UpdateResult::Updated {
                    dailies,
                    weekly,
                    gap,
                } => {
                    write!(
                        f,
                        "Updated {{ dailies: {}, weekly: {}, gap: {} }}",
                        dailies,
                        weekly,
                        gap.is_some()
                    )
                }
                UpdateResult::Blocked { gap } => {
                    write!(
                        f,
                        "Blocked {{ missing: {}, next: {} }}",
                        gap.missing_date, gap.next_available_date
                    )
                }
                UpdateResult::CheckOnly { available, gap } => {
                    write!(
                        f,
                        "CheckOnly {{ available: {}, gap: {} }}",
                        available,
                        gap.is_some()
                    )
                }
            }
        }
    }
}
