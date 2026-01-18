//! FRN lookup command - look up all licenses by FCC Registration Number.

use std::collections::HashSet;

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine};

use super::auto_update;

pub async fn execute(frns: &[String], service_override: &str, format: &str) -> Result<()> {
    if frns.is_empty() {
        anyhow::bail!("At least one FRN is required");
    }

    // FRN lookups can't auto-detect service, so use the override (defaulting to amateur)
    let service_code = auto_update::service_name_to_code(service_override)
        .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", service_override))?;

    // Ensure data is available, auto-download if needed
    let db = auto_update::ensure_data_available(service_code)
        .await
        .context("Failed to ensure data is available")?;

    let engine = QueryEngine::with_database(db);
    let output_format = format.parse::<OutputFormat>().unwrap_or_default();

    // Query each FRN, collect results
    let mut all_licenses = Vec::new();
    let mut not_found = Vec::new();

    for frn in frns {
        let licenses = engine.lookup_by_frn(frn)?;
        if licenses.is_empty() {
            not_found.push(frn.as_str());
        } else {
            all_licenses.extend(licenses);
        }
    }

    // Report not-found FRNs to stderr (non-blocking)
    for frn in &not_found {
        eprintln!("No licenses found for FRN: {}", frn);
    }

    // If nothing found at all, exit with error
    if all_licenses.is_empty() {
        std::process::exit(1);
    }

    // Deduplicate by unique_system_identifier (in case FRNs overlap)
    let seen: HashSet<i64> = HashSet::new();
    let mut deduped = Vec::new();
    let mut seen = seen;
    for license in all_licenses {
        if seen.insert(license.unique_system_identifier) {
            deduped.push(license);
        }
    }

    // Output all licenses using FormatOutput
    println!("{}", deduped.format(output_format));

    Ok(())
}
