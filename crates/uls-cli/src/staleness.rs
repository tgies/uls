//! Staleness checking and auto-update utilities for CLI commands.

use anyhow::Result;
use uls_db::{DataFreshness, Database, DatabaseConfig, DEFAULT_STALE_THRESHOLD_DAYS};

use crate::config::default_db_path;

/// Configuration for staleness behavior passed from CLI flags.
#[derive(Debug, Clone)]
pub struct StalenessOptions {
    /// Threshold in days before warning.
    pub threshold_days: i64,
    /// Whether warnings are enabled.
    pub warn_enabled: bool,
    /// Whether to auto-update on stale.
    pub auto_update: bool,
}

impl StalenessOptions {
    /// Create from CLI flags.
    pub fn from_cli(warn_stale: i64, no_stale_warning: bool, auto_update: bool) -> Self {
        Self {
            threshold_days: warn_stale,
            warn_enabled: !no_stale_warning,
            auto_update,
        }
    }

    /// Default options (3 days, warnings enabled, no auto-update).
    #[allow(dead_code)] // Reserved for future use
    pub fn default_options() -> Self {
        Self {
            threshold_days: DEFAULT_STALE_THRESHOLD_DAYS,
            warn_enabled: true,
            auto_update: false,
        }
    }
}

/// Check data freshness and optionally warn or auto-update.
///
/// Returns Ok(true) if the query should proceed, Ok(false) if it should abort.
#[allow(dead_code)] // Reserved for auto-update feature
pub async fn check_staleness(service_code: &str, options: &StalenessOptions) -> Result<bool> {
    let db_path = default_db_path();

    // If database doesn't exist yet, don't warn (they'll get an error about initializing)
    if !db_path.exists() {
        return Ok(true);
    }

    let config = DatabaseConfig::with_path(&db_path);
    let db = match Database::with_config(config) {
        Ok(db) => db,
        Err(_) => return Ok(true), // Let the actual command handle DB errors
    };

    // Check if initialized
    if !db.is_initialized().unwrap_or(false) {
        return Ok(true);
    }

    let freshness = db.get_freshness(service_code, options.threshold_days)?;

    if freshness.is_stale {
        if options.auto_update {
            eprintln!(
                "⚠ Data is stale ({}). Running auto-update...",
                freshness.age_display
            );

            // Run update for the service
            let service_name = match service_code {
                "HA" | "HV" => "amateur",
                "ZA" => "gmrs",
                _ => return Ok(true),
            };

            if let Err(e) = crate::commands::update::execute(service_name, false, false).await {
                eprintln!("⚠ Auto-update failed: {}", e);
                // Continue with stale data
            } else {
                eprintln!();
            }
        } else if options.warn_enabled {
            print_staleness_warning(&freshness);
        }
    }

    Ok(true)
}

/// Print a staleness warning to stderr.
pub fn print_staleness_warning(freshness: &DataFreshness) {
    let service_name = match freshness.service.as_str() {
        "HA" | "HV" => "amateur",
        "ZA" => "gmrs",
        _ => &freshness.service,
    };

    eprintln!(
        "⚠ Data is {} old. Run 'uls update {}' to refresh.",
        freshness.age_display, service_name
    );
    eprintln!("  (Use --no-stale-warning to suppress, or --auto-update to update automatically)");
    eprintln!();
}

/// Check staleness after a query and print a warning if stale.
/// This is a lighter version that just prints a warning at the end of output.
pub fn warn_if_stale_after_query(service_code: &str, options: &StalenessOptions) -> Result<()> {
    if !options.warn_enabled || options.auto_update {
        return Ok(());
    }

    let db_path = default_db_path();
    if !db_path.exists() {
        return Ok(());
    }

    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config)?;

    if !db.is_initialized().unwrap_or(false) {
        return Ok(());
    }

    let freshness = db.get_freshness(service_code, options.threshold_days)?;

    if freshness.is_stale {
        eprintln!();
        print_staleness_warning(&freshness);
    }

    Ok(())
}
