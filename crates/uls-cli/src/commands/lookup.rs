//! Lookup command - look up licenses by callsign.

use std::collections::HashSet;

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;
use crate::staleness::{warn_if_stale_after_query, StalenessOptions};

/// All supported services for cross-service lookup
const ALL_SERVICES: &[&str] = &["HA", "ZA"];

pub async fn execute(
    callsigns: &[String],
    service_override: &str,
    all_services: bool,
    format: &str,
    staleness_opts: &StalenessOptions,
) -> Result<()> {
    if callsigns.is_empty() {
        anyhow::bail!("At least one callsign is required");
    }

    // Batch service detection: collect unique services needed
    let services_needed: HashSet<&str> = callsigns
        .iter()
        .map(|cs| {
            if service_override == "auto" || service_override == "amateur" {
                auto_update::detect_service_from_callsign(cs)
            } else {
                auto_update::service_name_to_code(service_override).unwrap_or("HA")
            }
        })
        .collect();

    // Batch downloads: ensure each unique service is available
    for service in &services_needed {
        auto_update::ensure_data_available(service)
            .await
            .context(format!(
                "Failed to ensure data is available for {}",
                service
            ))?;
    }

    // Get database handle (any service will do, they share the same DB)
    let primary_service = services_needed.iter().next().unwrap_or(&"HA");
    let db = auto_update::ensure_data_available(primary_service).await?;
    let engine = QueryEngine::with_database(db);
    let output_format = format.parse::<OutputFormat>().unwrap_or_default();

    // Query each callsign, track results and failures
    let mut results = Vec::new();
    let mut not_found = Vec::new();

    for callsign in callsigns {
        match engine.lookup(callsign)? {
            Some(license) => results.push(license),
            None => not_found.push(callsign.as_str()),
        }
    }

    // Report not-found callsigns to stderr (non-blocking)
    for cs in &not_found {
        eprintln!("No license found for: {}", cs);
    }

    // If nothing found at all, exit with error
    if results.is_empty() {
        std::process::exit(1);
    }

    // If not requesting cross-service lookup, output what we have
    if !all_services {
        if callsigns.len() == 1 && results.len() == 1 {
            println!("{}", results[0].format(output_format));
        } else {
            println!("{}", results.format(output_format));
        }
        let _ = warn_if_stale_after_query(primary_service, staleness_opts);
        return Ok(());
    }

    // Cross-service lookup: collect unique FRNs from found licenses
    let frns: HashSet<String> = results
        .iter()
        .filter_map(|l| l.frn.clone())
        .filter(|f| !f.is_empty())
        .collect();

    if frns.is_empty() {
        // No FRNs available, can't do cross-service lookup
        eprintln!("\nNo FRNs available for cross-service lookup");
        println!("{}", results.format(output_format));
        return Ok(());
    }

    // Ensure ALL services are loaded for cross-service queries
    for &service in ALL_SERVICES {
        auto_update::ensure_data_available(service).await?;
    }

    // Re-get engine with all data loaded
    let db = auto_update::ensure_data_available(primary_service).await?;
    let engine = QueryEngine::with_database(db);

    // Query all FRNs and collect licenses
    let mut all_licenses = Vec::new();
    for frn in &frns {
        all_licenses.extend(engine.lookup_by_frn(frn)?);
    }

    // Deduplicate by unique_system_identifier
    all_licenses.sort_by(|a, b| a.unique_system_identifier.cmp(&b.unique_system_identifier));
    all_licenses.dedup_by(|a, b| a.unique_system_identifier == b.unique_system_identifier);

    // Sort so originally-requested callsigns appear first
    let requested: HashSet<&str> = callsigns.iter().map(|s| s.as_str()).collect();
    all_licenses.sort_by(|a, b| {
        let a_requested = requested.contains(a.call_sign.as_str());
        let b_requested = requested.contains(b.call_sign.as_str());
        b_requested.cmp(&a_requested) // requested ones sort first
    });

    // For single-callsign --all, use detailed format for each license
    // For multi-callsign, use collection format
    if callsigns.len() == 1 {
        for (i, license) in all_licenses.iter().enumerate() {
            if i > 0 {
                println!("\n---\n");
            }
            print!("{}", license.format(output_format));
        }
        println!(); // Final newline
    } else {
        println!("{}", all_licenses.format(output_format));
    }

    // Check staleness after query output
    let _ = warn_if_stale_after_query(primary_service, staleness_opts);

    Ok(())
}
