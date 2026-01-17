//! Search command - search for licenses.

use anyhow::{Context, Result};
use uls_query::{FormatOutput, OutputFormat, QueryEngine, SearchFilter, License};

use super::auto_update;

/// Format results with custom field selection.
fn format_with_fields(licenses: &[License], fields: &[&str], format: OutputFormat) -> String {
    match format {
        OutputFormat::Table => format_table_with_fields(licenses, fields),
        OutputFormat::Csv => format_csv_with_fields(licenses, fields),
        // For structured formats, just output all data
        _ => licenses.to_vec().format(format),
    }
}

fn format_table_with_fields(licenses: &[License], fields: &[&str]) -> String {
    if licenses.is_empty() {
        return "No results found.\n".to_string();
    }
    
    // Calculate column widths (min 6, max 30)
    let widths: Vec<usize> = fields.iter()
        .map(|f| f.len().max(6).min(30))
        .collect();
    
    let mut output = String::new();
    
    // Header
    for (i, field) in fields.iter().enumerate() {
        output.push_str(&format!("{:width$} ", field, width = widths[i]));
    }
    output.push('\n');
    
    // Separator
    for width in &widths {
        output.push_str(&format!("{:-<width$} ", "", width = *width));
    }
    output.push('\n');
    
    // Rows
    for license in licenses {
        for (i, field) in fields.iter().enumerate() {
            let value = license.get_field(field).unwrap_or_else(|| "-".to_string());
            let truncated = if value.len() > widths[i] {
                format!("{}...", &value[..widths[i].saturating_sub(3)])
            } else {
                value
            };
            output.push_str(&format!("{:width$} ", truncated, width = widths[i]));
        }
        output.push('\n');
    }
    
    output.push_str(&format!("\n{} result(s)\n", licenses.len()));
    output
}

fn format_csv_with_fields(licenses: &[License], fields: &[&str]) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str(&fields.join(","));
    output.push('\n');
    
    // Rows
    for license in licenses {
        let values: Vec<String> = fields.iter()
            .map(|f| {
                let v = license.get_field(f).unwrap_or_default();
                if v.contains(',') || v.contains('"') || v.contains('\n') {
                    format!("\"{}\"", v.replace('"', "\"\""))
                } else {
                    v
                }
            })
            .collect();
        output.push_str(&values.join(","));
        output.push('\n');
    }
    
    output
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    query: Option<String>,
    name: Option<String>,
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
    fields: Option<String>,
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
        || name.is_some()
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
        eprintln!("Quick examples:");
        eprintln!("  uls search Smith -c Denver -r gmrs   # Find Smith in Denver (GMRS)");
        eprintln!("  uls search -n Smith -s TX          # Name search in Texas");
        eprintln!("  uls search W1* -a                  # Active callsigns starting with W1");
        eprintln!("  uls search -s FL -S=-grant_date    # Recent grants in Florida");
        eprintln!();
        eprintln!("Run 'uls search --help' for all options.");
        std::process::exit(1);
    }

    // Build filter from convenience args
    // Priority: explicit -n/--name > positional query
    // Auto-add wildcards to name for partial matching if not already present
    let mut filter = if let Some(ref n) = name {
        let name_pattern = if n.contains('*') || n.contains('?') {
            n.clone()
        } else {
            format!("*{}*", n) // Partial match by default
        };
        SearchFilter::name(&name_pattern)
    } else if let Some(ref q) = query {
        // Positional query is always a name search (use `lookup` for callsigns)
        let name_pattern = if q.contains('*') || q.contains('?') {
            q.clone()
        } else {
            format!("*{}*", q) // Partial match by default
        };
        SearchFilter::name(&name_pattern)
    } else {
        SearchFilter::new()
    };

    if let Some(s) = state {
        filter = filter.with_state(s);
    }

    if let Some(c) = city {
        // Use generic filter for LIKE matching (handles partial/case matches)
        filter = filter.with_filter(format!("city={}", c));
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

    // Apply service filter - the unified DB contains all services
    // Map the service code to the appropriate radio service codes
    filter.radio_service = Some(match service_code {
        "HA" => vec!["HA".to_string(), "HV".to_string()], // Amateur includes HV (vanity)
        "ZA" => vec!["ZA".to_string()], // GMRS
        _ => vec![service_code.to_string()],
    });

    let results = engine.search(filter)?;

    if results.is_empty() {
        eprintln!("No results found.");
        std::process::exit(1);
    }

    // Output with custom fields if specified
    if let Some(ref field_list) = fields {
        let field_vec: Vec<&str> = field_list.split(',').map(|s| s.trim()).collect();
        println!("{}", format_with_fields(&results, &field_vec, output_format));
    } else {
        println!("{}", results.format(output_format));
    }
    
    Ok(())
}
