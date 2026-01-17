//! Lookup command - look up a license by callsign.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;

pub async fn execute(callsign: &str, format: &str) -> Result<()> {
    // Default to amateur service for callsign lookups
    let service = auto_update::service_name_to_code("amateur")
        .expect("amateur is a valid service");
    
    // Ensure data is available, auto-download if needed
    let db = auto_update::ensure_data_available(service).await
        .context("Failed to ensure data is available")?;

    let engine = QueryEngine::with_database(db);
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
