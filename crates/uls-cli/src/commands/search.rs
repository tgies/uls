//! Search command - search for licenses.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine, SearchFilter};

/// Get the default database path.
fn default_db_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("uls")
        .join("uls.db")
}

pub async fn execute(
    query: Option<String>,
    state: Option<String>,
    city: Option<String>,
    class: Option<char>,
    active: bool,
    limit: usize,
    format: &str,
) -> Result<()> {
    let db_path = default_db_path();
    
    let engine = QueryEngine::open(&db_path)
        .context("Failed to open database. Run 'uls update' first to initialize.")?;

    let output_format = OutputFormat::from_str(format).unwrap_or_default();

    // Build filter
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

    if let Some(c) = class {
        filter = filter.with_operator_class(c.to_ascii_uppercase());
    }

    if active {
        filter = filter.active_only();
    }

    filter = filter.with_limit(limit);

    let results = engine.search(filter)?;

    if results.is_empty() {
        eprintln!("No results found.");
        std::process::exit(1);
    }

    println!("{}", results.format(output_format));
    Ok(())
}
