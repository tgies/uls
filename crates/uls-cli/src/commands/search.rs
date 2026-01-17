//! Search command - search for licenses.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine, SearchFilter};

use super::auto_update;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    query: Option<String>,
    state: Option<String>,
    city: Option<String>,
    zip: Option<String>,
    frn: Option<String>,
    class: Option<char>,
    status: Option<char>,
    active: bool,
    granted_after: Option<String>,
    granted_before: Option<String>,
    expires_before: Option<String>,
    filters: Vec<String>,
    sort: &str,
    limit: usize,
    service_override: &str,
    format: &str,
) -> Result<()> {
    // Use service override (defaulting to amateur for searches)
    let service_code = auto_update::service_name_to_code(service_override)
        .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", service_override))?;
    
    // Ensure data is available, auto-download if needed
    let db = auto_update::ensure_data_available(service_code).await
        .context("Failed to ensure data is available")?;

    let engine = QueryEngine::with_database(db);
    let output_format = OutputFormat::from_str(format).unwrap_or_default();

    // Require at least one search filter
    let has_filter = query.is_some() 
        || state.is_some() 
        || city.is_some() 
        || zip.is_some()
        || frn.is_some()
        || class.is_some() 
        || status.is_some()
        || active
        || granted_after.is_some()
        || granted_before.is_some()
        || expires_before.is_some()
        || !filters.is_empty();

    if !has_filter {
        eprintln!("Error: At least one search filter is required.");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  uls search W1*                          # Callsign pattern");
        eprintln!("  uls search \"John Smith\"                 # Name search");
        eprintln!("  uls search --state TX                   # By state");
        eprintln!("  uls search --filter \"grant_date>2025-01-01\"  # Generic filter");
        eprintln!("  uls search --active --sort -grant_date  # Recent grants");
        std::process::exit(1);
    }

    // Build filter from convenience args
    let mut filter = if let Some(ref q) = query {
        if q.contains('*') || q.contains('?') {
            SearchFilter::callsign(q)
        } else if q.chars().all(|c| c.is_alphanumeric()) && q.len() <= 10 {
            // Looks like a callsign
            SearchFilter::callsign(q)
        } else {
            SearchFilter::name(q)
        }
    } else {
        SearchFilter::new()
    };

    if let Some(s) = state {
        filter = filter.with_state(s);
    }

    if let Some(c) = city {
        filter.city = Some(c.to_uppercase());
    }

    if let Some(z) = zip {
        filter.zip_code = Some(z);
    }

    if let Some(f) = frn {
        filter.frn = Some(f);
    }

    if let Some(c) = class {
        filter = filter.with_operator_class(c.to_ascii_uppercase());
    }

    if let Some(s) = status {
        filter.status = Some(s.to_ascii_uppercase());
    } else if active {
        filter = filter.active_only();
    }

    // Date filters (legacy convenience)
    filter.granted_after = granted_after;
    filter.granted_before = granted_before;
    filter.expires_before = expires_before;

    // Generic filters
    for f in filters {
        filter = filter.with_filter(&f);
    }

    // Sort by field (supports -field for descending)
    filter = filter.with_sort_field(sort);
    filter = filter.with_limit(limit);

    let results = engine.search(filter)?;

    if results.is_empty() {
        eprintln!("No results found.");
        std::process::exit(1);
    }

    println!("{}", results.format(output_format));
    Ok(())
}
