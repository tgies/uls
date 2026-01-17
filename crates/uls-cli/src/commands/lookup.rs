//! Lookup command - look up a license by callsign.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

/// Get the default database path.
fn default_db_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

pub async fn execute(callsign: &str, format: &str) -> Result<()> {
    let db_path = default_db_path();
    
    let engine = QueryEngine::open(&db_path)
        .context("Failed to open database. Run 'uls update' first to initialize.")?;

    let output_format = OutputFormat::from_str(format).unwrap_or_default();

    match engine.lookup(callsign)? {
        Some(license) => {
            println!("{}", license.format(output_format));
            Ok(())
        }
        None => {
            eprintln!("No license found for callsign: {}", callsign.to_uppercase());
            std::process::exit(1);
        }
    }
}
