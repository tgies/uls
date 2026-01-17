//! ULS CLI tool for FCC data management.
//!
//! Provides commands for downloading, updating, and querying FCC ULS data.
//!
//! # Quick Lookup
//!
//! You can look up a callsign directly:
//! ```sh
//! uls W1AW        # equivalent to: uls lookup W1AW
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

#[derive(Parser)]
#[command(name = "uls")]
#[command(author, version, about = "FCC Universal Licensing System data tool", long_about = None)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Output format (table, json, csv, yaml)
    #[arg(short, long, default_value = "table", global = true)]
    format: String,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Callsign for quick lookup (shorthand for 'uls lookup <CALLSIGN>')
    #[arg(value_name = "CALLSIGN")]
    callsign: Option<String>,

    /// Also show licenses from other services for the same FRN (with shorthand lookup)
    #[arg(short, long)]
    all: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Look up a license by callsign
    Lookup {
        /// Callsign to look up
        callsign: String,

        /// Radio service (amateur, gmrs)
        #[arg(long, default_value = "amateur")]
        service: String,

        /// Also show licenses from other services for the same FRN
        #[arg(short, long)]
        all: bool,
    },

    /// Search for licenses
    Search {
        /// Search query (name, callsign pattern, etc.)
        query: Option<String>,

        /// Filter by state (2-letter code)
        #[arg(short, long)]
        state: Option<String>,

        /// Filter by city
        #[arg(long)]
        city: Option<String>,

        /// Filter by ZIP code
        #[arg(long)]
        zip: Option<String>,

        /// Filter by FRN
        #[arg(long)]
        frn: Option<String>,

        /// Filter by operator class (T, G, A, E)
        #[arg(short, long)]
        class: Option<char>,

        /// Filter by license status (A=Active, E=Expired, C=Cancelled)
        #[arg(long)]
        status: Option<char>,

        /// Only show active licenses (shortcut for --status A)
        #[arg(long)]
        active: bool,

        /// Licenses granted on or after this date (YYYY-MM-DD)
        #[arg(long)]
        granted_after: Option<String>,

        /// Licenses granted on or before this date (YYYY-MM-DD)
        #[arg(long)]
        granted_before: Option<String>,

        /// Licenses expiring on or before this date (YYYY-MM-DD)
        #[arg(long)]
        expires_before: Option<String>,

        /// Generic filter expressions (repeatable, e.g., --filter "grant_date>2025-01-01")
        #[arg(long = "filter", short = 'F')]
        filters: Vec<String>,

        /// Sort order: callsign, -callsign, name, state, granted, expires
        #[arg(long, default_value = "callsign")]
        sort: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Radio service (amateur, gmrs)
        #[arg(long, default_value = "amateur")]
        service: String,
    },

    /// Update the local database
    Update {
        /// Radio service to update (amateur, gmrs, all)
        #[arg(short, long, default_value = "amateur")]
        service: String,

        /// Force full download even if cached
        #[arg(long)]
        force: bool,

        /// Import only HD+EN+AM records (sufficient for callsign/FRN lookups)
        #[arg(long)]
        minimal: bool,
    },

    /// Look up all licenses by FRN (FCC Registration Number)
    Frn {
        /// FRN to look up (10-digit FCC Registration Number)
        frn: String,

        /// Radio service (amateur, gmrs)
        #[arg(long, default_value = "amateur")]
        service: String,
    },

    /// Show database statistics
    Stats,

    /// Manage the database
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Initialize a new database
    Init {
        /// Path to database file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Show database info
    Info,

    /// Vacuum/optimize the database
    Vacuum,
}

/// Check if a string looks like a callsign.
fn looks_like_callsign(s: &str) -> bool {
    let s = s.to_uppercase();
    // Callsign pattern: 1-2 letters, 1 digit, 1-4 letters (US amateur)
    // Examples: W1AW, K1ABC, N1MM, AA1A, KA1AAA
    if s.len() < 3 || s.len() > 7 {
        return false;
    }
    
    let chars: Vec<char> = s.chars().collect();
    
    // Must start with letter
    if !chars[0].is_ascii_alphabetic() {
        return false;
    }
    
    // Must contain at least one digit
    if !chars.iter().any(|c| c.is_ascii_digit()) {
        return false;
    }
    
    // All chars must be alphanumeric
    chars.iter().all(|c| c.is_ascii_alphanumeric())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .init();

    // Execute command
    match cli.command {
        Some(Commands::Lookup { callsign, service, all }) => {
            commands::lookup::execute(&callsign, &service, all, &cli.format).await
        }
        Some(Commands::Search {
            query,
            state,
            city,
            zip,
            frn,
            class,
            status,
            active,
            granted_after,
            granted_before,
            expires_before,
            filters,
            sort,
            limit,
            service,
        }) => {
            commands::search::execute(
                query, state, city, zip, frn, class, status, active,
                granted_after, granted_before, expires_before, filters,
                &sort, limit, &service, &cli.format
            ).await
        }
        Some(Commands::Update { service, force, minimal }) => {
            commands::update::execute(&service, force, minimal).await
        }
        Some(Commands::Frn { frn, service }) => {
            commands::frn::execute(&frn, &service, &cli.format).await
        }
        Some(Commands::Stats) => {
            commands::stats::execute(&cli.format).await
        }
        Some(Commands::Db { command }) => match command {
            DbCommands::Init { path } => commands::db::init(path).await,
            DbCommands::Info => commands::db::info(&cli.format).await,
            DbCommands::Vacuum => commands::db::vacuum().await,
        },
        None => {
            // No subcommand - check for quick callsign lookup
            if let Some(callsign) = cli.callsign {
                if looks_like_callsign(&callsign) {
                    commands::lookup::execute(&callsign, "amateur", cli.all, &cli.format).await
                } else {
                    eprintln!("Unknown command or invalid callsign: {}", callsign);
                    eprintln!("Run 'uls --help' for usage information.");
                    std::process::exit(1);
                }
            } else {
                // No command and no callsign - show help
                use clap::CommandFactory;
                Cli::command().print_help()?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_callsign() {
        assert!(looks_like_callsign("W1AW"));
        assert!(looks_like_callsign("K1ABC"));
        assert!(looks_like_callsign("N1MM"));
        assert!(looks_like_callsign("AA1A"));
        assert!(looks_like_callsign("w1aw")); // case insensitive
        assert!(looks_like_callsign("KA1AAA"));
        
        assert!(!looks_like_callsign("lookup")); // no digits
        assert!(!looks_like_callsign("search")); // no digits
        assert!(!looks_like_callsign("AB"));     // too short
        assert!(!looks_like_callsign("")); 
        assert!(!looks_like_callsign("12345"));  // no letters at start
    }
}
