//! FRN lookup command - look up all licenses by FCC Registration Number.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

/// Get the default database path.
fn default_db_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

pub async fn execute(frn: &str, format: &str) -> Result<()> {
    let db_path = default_db_path();
    
    let engine = QueryEngine::open(&db_path)
        .context("Failed to open database. Run 'uls update' first to initialize.")?;

    let output_format = OutputFormat::from_str(format).unwrap_or_default();
    
    let licenses = engine.lookup_by_frn(frn)?;
    
    if licenses.is_empty() {
        eprintln!("No licenses found for FRN: {}", frn);
        std::process::exit(1);
    }
    
    // Print header with FRN info
    println!("FRN {} - {} license(s):\n", frn, licenses.len());
    
    // Print each license
    for (i, license) in licenses.iter().enumerate() {
        if i > 0 {
            println!("---");
        }
        println!("{}", license.format(output_format));
    }
    
    Ok(())
}
