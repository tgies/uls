//! Lookup command - look up a license by callsign.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;

/// All supported services for cross-service lookup
const ALL_SERVICES: &[&str] = &["HA", "ZA"];

pub async fn execute(callsign: &str, service_override: &str, all_services: bool, format: &str) -> Result<()> {
    // Use service override if provided, otherwise auto-detect from callsign
    let primary_service = if service_override == "auto" || service_override == "amateur" {
        // For "amateur" default, still auto-detect to catch GMRS
        auto_update::detect_service_from_callsign(callsign)
    } else {
        auto_update::service_name_to_code(service_override)
            .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", service_override))?
    };
    
    // Ensure primary service data is available
    let db = auto_update::ensure_data_available(primary_service).await
        .context("Failed to ensure data is available")?;

    let engine = QueryEngine::with_database(db);
    let output_format = OutputFormat::from_str(format).unwrap_or_default();

    // Look up the primary license
    let primary_license = match engine.lookup(callsign)? {
        Some(license) => license,
        None => {
            eprintln!("No license found for callsign: {}", callsign.to_uppercase());
            std::process::exit(1);
        }
    };

    // If not requesting all services, just print the primary and exit
    if !all_services {
        println!("{}", primary_license.format(output_format));
        return Ok(());
    }

    // Cross-service lookup: need FRN
    let frn = match &primary_license.frn {
        Some(f) if !f.is_empty() => f.clone(),
        _ => {
            // No FRN, can't do cross-service lookup
            println!("{}", primary_license.format(output_format));
            eprintln!("\nNo FRN available for cross-service lookup");
            return Ok(());
        }
    };

    // Ensure ALL services are loaded into the single database
    for &service in ALL_SERVICES {
        if service == primary_service {
            continue; // Already loaded
        }
        auto_update::ensure_data_available(service).await?;
    }

    // Now query by FRN - this will find all licenses across all services in the single DB
    let db = auto_update::ensure_data_available(primary_service).await?;
    let engine = QueryEngine::with_database(db);
    let mut all_licenses = engine.lookup_by_frn(&frn)?;

    // Ensure the originally looked-up callsign appears first
    let callsign_upper = callsign.to_uppercase();
    all_licenses.sort_by(|a, b| {
        let a_is_primary = a.call_sign.to_uppercase() == callsign_upper;
        let b_is_primary = b.call_sign.to_uppercase() == callsign_upper;
        b_is_primary.cmp(&a_is_primary) // true sorts before false
    });

    // Output all licenses using FormatOutput (handles JSON properly)
    println!("{}", all_licenses.format(output_format));

    Ok(())
}
