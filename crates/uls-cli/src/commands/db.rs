//! Database management commands.

use anyhow::{Context, Result};
use uls_db::{Database, DatabaseConfig};
use uls_query::{OutputFormat, QueryEngine};

use crate::config::default_db_path;

/// Initialize a new database.
pub async fn init(path: Option<String>) -> Result<()> {
    let db_path = path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_db_path);

    println!("Initializing database at: {}", db_path.display());

    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;
    db.initialize()?;

    println!("✓ Database initialized successfully.");
    Ok(())
}

/// Show database info.
pub async fn info(format: &str) -> Result<()> {
    let db_path = default_db_path();

    if !db_path.exists() {
        eprintln!("Database not found at: {}", db_path.display());
        eprintln!("Run 'uls db init' to create a new database.");
        std::process::exit(1);
    }

    let engine = QueryEngine::open(&db_path).context("Failed to open database")?;

    let stats = engine.stats()?;
    let output_format = format.parse::<OutputFormat>().unwrap_or_default();

    match output_format {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let info = serde_json::json!({
                "path": db_path.display().to_string(),
                "size_bytes": std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0),
                "initialized": engine.is_ready().unwrap_or(false),
                "schema_version": stats.schema_version,
                "total_licenses": stats.total_licenses,
                "last_updated": stats.last_updated,
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        _ => {
            let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
            println!("Database Info");
            println!("=============");
            println!("Path:           {}", db_path.display());
            println!("Size:           {} bytes", size);
            println!("Initialized:    {}", engine.is_ready().unwrap_or(false));
            println!("Schema version: {}", stats.schema_version);
            println!("Total licenses: {}", stats.total_licenses);
            if let Some(ref updated) = stats.last_updated {
                println!("Last updated:   {}", updated);
            }
        }
    }

    Ok(())
}

/// Vacuum/optimize the database.
pub async fn vacuum() -> Result<()> {
    let db_path = default_db_path();

    if !db_path.exists() {
        eprintln!("Database not found at: {}", db_path.display());
        std::process::exit(1);
    }

    println!("Optimizing database...");

    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;
    let conn = db.conn()?;

    conn.execute_batch("VACUUM; ANALYZE;")?;

    println!("✓ Database optimized.");
    Ok(())
}
