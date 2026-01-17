//! FRN lookup command - look up all licenses by FCC Registration Number.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;

pub async fn execute(frn: &str, service_override: &str, format: &str) -> Result<()> {
    // FRN lookups can't auto-detect, so use the service override (defaulting to amateur)
    let service_code = auto_update::service_name_to_code(service_override)
        .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", service_override))?;
    
    // Ensure data is available, auto-download if needed
    let db = auto_update::ensure_data_available(service_code).await
        .context("Failed to ensure data is available")?;

    let engine = QueryEngine::with_database(db);
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
