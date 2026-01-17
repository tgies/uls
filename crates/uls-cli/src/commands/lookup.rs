//! Lookup command - look up a license by callsign.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;

pub async fn execute(callsign: &str, service_override: &str, format: &str) -> Result<()> {
    // Use service override if provided, otherwise auto-detect from callsign
    let service_code = if service_override == "auto" || service_override == "amateur" {
        // For "amateur" default, still auto-detect to catch GMRS
        auto_update::detect_service_from_callsign(callsign)
    } else {
        auto_update::service_name_to_code(service_override)
            .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", service_override))?
    };
    
    // Ensure data is available, auto-download if needed
    let db = auto_update::ensure_data_available(service_code).await
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
