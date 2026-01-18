//! Stats command - show database statistics.

use anyhow::{Context, Result};
use uls_query::{OutputFormat, QueryEngine};

use crate::config::default_db_path;

pub async fn execute(format: &str) -> Result<()> {
    let db_path = default_db_path();

    let engine = QueryEngine::open(&db_path)
        .context("Failed to open database. Run 'uls update' first to initialize.")?;

    let stats = engine.stats()?;
    let output_format = format.parse::<OutputFormat>().unwrap_or_default();

    match output_format {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::to_string_pretty(&stats)?;
            println!("{}", json);
        }
        _ => {
            println!("Database Statistics");
            println!("===================");
            println!("Total licenses:     {:>10}", stats.total_licenses);
            println!("Active licenses:    {:>10}", stats.active_licenses);
            println!("Expired licenses:   {:>10}", stats.expired_licenses);
            println!("Cancelled licenses: {:>10}", stats.cancelled_licenses);
            println!();
            if let Some(ref updated) = stats.last_updated {
                println!("Last updated: {}", updated);
            }
            println!("Schema version: {}", stats.schema_version);
        }
    }

    Ok(())
}
