//! Import fixed local archives into a new disposable database, without downloads.
//!
//! Usage: benchmark_import OUTPUT_DIR full|minimal count-first|stream|batch HA=ZIP ZA=ZIP [SEED=DB]
//! The output directory must not exist. Timings exclude subsequent verification.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use uls_db::{Database, ImportBatch, ImportMode, Importer};
use uls_parser::archive::ZipExtractor;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .without_time()
        .init();
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 4 {
        return Err(
            "usage: benchmark_import OUTPUT_DIR full|minimal count-first|stream|batch HA=ZIP [ZA=ZIP] [SEED=DB]"
                .into(),
        );
    }
    let output = PathBuf::from(&args[0]);
    let mode = match args[1].as_str() {
        "full" => ImportMode::Full,
        "minimal" => ImportMode::Minimal,
        _ => return Err("expected full or minimal import mode".into()),
    };
    let count_first = match args[2].as_str() {
        "count-first" => true,
        "stream" | "batch" => false,
        _ => return Err("expected count-first, stream or batch".into()),
    };
    let seeds: Vec<_> = args[3..]
        .iter()
        .filter_map(|arg| arg.strip_prefix("SEED="))
        .collect();
    if seeds.len() > 1 {
        return Err("only one seed database is supported".into());
    }
    let archives = args[3..]
        .iter()
        .filter(|arg| !arg.starts_with("SEED="))
        .map(|arg| {
            let (service, path) = arg.split_once('=').ok_or("expected SERVICE=ZIP")?;
            if !matches!(service, "HA" | "ZA" | "HA_DAILY" | "ZA_DAILY")
                || !Path::new(path).is_file()
            {
                return Err("expected HA, ZA, HA_DAILY or ZA_DAILY and an existing ZIP file");
            }
            Ok((service, Path::new(path)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if archives.is_empty() {
        return Err("at least one archive is required".into());
    }

    std::fs::create_dir(&output)?;
    let copy_start = Instant::now();
    if let Some(seed) = seeds.first() {
        for suffix in ["-wal", "-journal"] {
            let sidecar = PathBuf::from(format!("{seed}{suffix}"));
            if sidecar.try_exists()? && sidecar.metadata()?.len() != 0 {
                return Err(
                    format!("seed has an uncheckpointed journal: {}", sidecar.display()).into(),
                );
            }
        }
        // Keep the disposable output writable even when the source is read-only.
        std::io::copy(
            &mut std::fs::File::open(seed)?,
            &mut std::fs::File::create_new(output.join("import.db"))?,
        )?;
    }
    let seed_copy_seconds = copy_start.elapsed().as_secs_f64();
    let start = Instant::now();
    let db = Database::open(output.join("import.db"))?;
    db.initialize()?;
    let initialize_seconds = start.elapsed().as_secs_f64();
    let reports = if args[2] == "batch" {
        Importer::new(&db).batch(|batch| -> Result<Vec<_>, Box<dyn Error>> {
            archives
                .iter()
                .map(|(service, path)| run_archive(batch, service, path, &mode, count_first))
                .collect()
        })?
    } else {
        let mut reports = Vec::new();
        for (service, path) in &archives {
            reports.push(
                Importer::new(&db)
                    .batch(|batch| run_archive(batch, service, path, &mode, count_first))?,
            );
        }
        reports
    };
    drop(db);
    let elapsed_seconds = start.elapsed().as_secs_f64();
    let report = json!({
        "mode": args[1],
        "count_first": count_first,
        "pipelined_parsing": false,
        "combined_indexes": args[2] == "batch",
        "archive_timing_scope": "per-archive processing; batch restoration included in elapsed_seconds",
        "seeded": !seeds.is_empty(),
        "seed_copy_seconds": seed_copy_seconds,
        "initialize_seconds": initialize_seconds,
        "elapsed_seconds": elapsed_seconds,
        "archives": reports,
    });
    std::fs::write(output.join("report.json"), format!("{report}\n"))?;
    println!("{report}");
    Ok(())
}

fn run_archive(
    batch: &mut ImportBatch<'_>,
    service: &str,
    path: &Path,
    mode: &ImportMode,
    count_first: bool,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let count_start = Instant::now();
    let counts = if count_first {
        Some(ZipExtractor::open(path)?.count_all_records()?)
    } else {
        None
    };
    let count_seconds = count_start.elapsed().as_secs_f64();
    let import_start = Instant::now();
    let stats = if service.ends_with("_DAILY") {
        batch.import_patch(path, mode.clone(), None)?
    } else {
        batch.import_for_service(path, service, mode.clone(), None)?
    };
    let import_seconds = import_start.elapsed().as_secs_f64();
    let report = json!({
        "service": service,
        "archive": path,
        "counts": counts,
        "count_seconds": count_seconds,
        "import_seconds": import_seconds,
        "records": stats.records,
        "files": stats.files,
        "parse_errors": stats.parse_errors,
        "insert_errors": stats.insert_errors,
    });
    eprintln!("{report}");
    Ok(report)
}
