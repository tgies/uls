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
}

#[derive(Subcommand)]
enum Commands {
    /// Look up a license by callsign
    Lookup {
        /// Callsign to look up
        callsign: String,
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

        /// Filter by operator class (T, G, A, E)
        #[arg(short, long)]
        class: Option<char>,

        /// Only show active licenses
        #[arg(long)]
        active: bool,

        /// Maximum results to return
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Update the local database
    Update {
        /// Radio service to update (amateur, gmrs, all)
        #[arg(short, long, default_value = "amateur")]
        service: String,

        /// Force full download even if cached
        #[arg(long)]
        force: bool,

        /// Skip daily incremental updates
        #[arg(long)]
        full_only: bool,
    },

    /// Look up all licenses by FRN (FCC Registration Number)
    Frn {
        /// FRN to look up (10-digit FCC Registration Number)
        frn: String,
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
        Some(Commands::Lookup { callsign }) => {
            commands::lookup::execute(&callsign, &cli.format).await
        }
        Some(Commands::Search {
            query,
            state,
            city,
            class,
            active,
            limit,
        }) => {
            commands::search::execute(query, state, city, class, active, limit, &cli.format).await
        }
        Some(Commands::Update { service, force, full_only }) => {
            commands::update::execute(&service, force, full_only).await
        }
        Some(Commands::Frn { frn }) => {
            commands::frn::execute(&frn, &cli.format).await
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
                    commands::lookup::execute(&callsign, &cli.format).await
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
