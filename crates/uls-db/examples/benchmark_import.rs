//! Import fixed local archives into a new disposable database, without downloads.
//!
//! Usage: benchmark_import OUTPUT_DIR full|minimal count-first|stream|pipeline HA=ZIP ZA=ZIP [SEED=DB]
//! The output directory must not exist. Timings exclude subsequent verification.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use uls_db::{Database, ImportMode, Importer};
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
            "usage: benchmark_import OUTPUT_DIR full|minimal count-first|stream|pipeline HA=ZIP [ZA=ZIP] [SEED=DB]"
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
        "stream" | "pipeline" => false,
        _ => return Err("expected count-first, stream or pipeline".into()),
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
            if !matches!(service, "HA" | "ZA") || !Path::new(path).is_file() {
                return Err("expected HA or ZA and an existing ZIP file");
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
    let mut reports = Vec::new();
    for (service, path) in archives {
        let count_start = Instant::now();
        let counts = if count_first {
            Some(ZipExtractor::open(path)?.count_all_records()?)
        } else {
            None
        };
        let count_seconds = count_start.elapsed().as_secs_f64();
        let import_start = Instant::now();
        let stats = Importer::new(&db)
            .with_pipelined_parsing(args[2] == "pipeline")
            .import_for_service(path, service, mode.clone(), None)?;
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
        reports.push(report);
    }
    drop(db);
    let elapsed_seconds = start.elapsed().as_secs_f64();
    let report = json!({
        "mode": args[1],
        "count_first": count_first,
        "pipelined_parsing": args[2] == "pipeline",
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
