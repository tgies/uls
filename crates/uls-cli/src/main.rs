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
mod config;
mod staleness;

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

    /// Staleness threshold in days (warn if data older than this)
    #[arg(long, default_value = "3", global = true, value_name = "DAYS")]
    warn_stale: i64,

    /// Disable staleness warnings
    #[arg(long, global = true)]
    no_stale_warning: bool,

    /// Automatically update stale data before queries
    #[arg(long, global = true)]
    auto_update: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Callsigns for quick lookup (shorthand for `uls lookup <CALLSIGN>...`)
    #[arg(value_name = "CALLSIGN")]
    callsigns: Vec<String>,

    /// Also show licenses from other services for the same FRN (with shorthand lookup)
    #[arg(short, long)]
    all: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Look up licenses by callsign (one or more)
    Lookup {
        /// Callsigns to look up
        #[arg(required = true)]
        callsigns: Vec<String>,

        /// Radio service (amateur, gmrs, auto)
        #[arg(short = 'r', long, default_value = "auto")]
        service: String,

        /// Also show licenses from other services for the same FRN
        #[arg(short, long)]
        all: bool,
    },

    /// Search for licenses
    Search(Box<SearchArgs>),

    /// Update the local database
    Update {
        /// Radio service to update (amateur, gmrs, all)
        #[arg(short = 'r', long, default_value = "amateur")]
        service: String,

        /// Force full download even if cached
        #[arg(long)]
        force: bool,

        /// Import only HD+EN+AM records (sufficient for callsign/FRN lookups)
        #[arg(long)]
        minimal: bool,

        /// Only apply daily patches (skip weekly check)
        #[arg(long)]
        daily_only: bool,

        /// Check for available updates without downloading
        #[arg(long)]
        check: bool,
    },

    /// Look up all licenses by FRN (FCC Registration Number)
    Frn {
        /// FRNs to look up (10-digit FCC Registration Numbers)
        #[arg(required = true)]
        frns: Vec<String>,

        /// Radio service (amateur, gmrs)
        #[arg(short = 'r', long, default_value = "amateur")]
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

/// Arguments for the search command (boxed to reduce enum size)
#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search query (name, callsign pattern, etc.)
    pub query: Option<String>,

    /// Name search (explicit, vs positional which auto-detects)
    #[arg(short = 'n', long)]
    pub name: Option<String>,

    /// Filter by state (2-letter code)
    #[arg(short, long)]
    pub state: Option<String>,

    /// Filter by city
    #[arg(short = 'c', long)]
    pub city: Option<String>,

    /// Filter by ZIP code
    #[arg(long)]
    pub zip: Option<String>,

    /// Filter by FRN
    #[arg(long)]
    pub frn: Option<String>,

    /// Filter by operator class (T, G, A, E)
    #[arg(short = 'C', long)]
    pub class: Option<char>,

    /// Filter by license status (A=Active, E=Expired, C=Cancelled)
    #[arg(long)]
    pub status: Option<char>,

    /// Only show active licenses (shortcut for --status A)
    #[arg(short = 'a', long)]
    pub active: bool,

    /// Licenses granted on or after this date (YYYY-MM-DD)
    #[arg(long)]
    pub granted_after: Option<String>,

    /// Licenses granted on or before this date (YYYY-MM-DD)
    #[arg(long)]
    pub granted_before: Option<String>,

    /// Licenses expiring on or before this date (YYYY-MM-DD)
    #[arg(long)]
    pub expires_before: Option<String>,

    /// Generic filter expressions (repeatable, e.g., --filter "grant_date>2025-01-01")
    #[arg(long = "filter", short = 'F')]
    pub filters: Vec<String>,

    /// Sort order: callsign, -callsign, name, state, granted, expires
    #[arg(short = 'S', long, default_value = "callsign")]
    pub sort: String,

    /// Maximum results to return
    #[arg(short, long, default_value = "50")]
    pub limit: usize,

    /// Radio service (amateur, gmrs)
    #[arg(short = 'r', long, default_value = "amateur")]
    pub service: String,

    /// Output fields (comma-separated, e.g., call_sign,name,grant_date)
    #[arg(long)]
    pub fields: Option<String>,
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
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .init();

    // Create staleness options from CLI flags
    let staleness_opts = staleness::StalenessOptions::from_cli(
        cli.warn_stale,
        cli.no_stale_warning,
        cli.auto_update,
    );

    // Execute command
    match cli.command {
        Some(Commands::Lookup {
            callsigns,
            service,
            all,
        }) => {
            commands::lookup::execute(&callsigns, &service, all, &cli.format, &staleness_opts).await
        }
        Some(Commands::Search(args)) => {
            commands::search::execute(
                args.query,
                args.name,
                args.state,
                args.city,
                args.zip,
                args.frn,
                args.class,
                args.status,
                args.active,
                args.granted_after,
                args.granted_before,
                args.expires_before,
                args.filters,
                &args.sort,
                args.limit,
                &args.service,
                &cli.format,
                args.fields,
            )
            .await
        }
        Some(Commands::Update {
            service,
            force,
            minimal,
            daily_only,
            check,
        }) => {
            commands::update::execute_with_options(&service, force, minimal, daily_only, check)
                .await
        }
        Some(Commands::Frn { frns, service }) => {
            commands::frn::execute(&frns, &service, &cli.format).await
        }
        Some(Commands::Stats) => commands::stats::execute(&cli.format).await,
        Some(Commands::Db { command }) => match command {
            DbCommands::Init { path } => commands::db::init(path).await,
            DbCommands::Info => commands::db::info(&cli.format).await,
            DbCommands::Vacuum => commands::db::vacuum().await,
        },
        None => {
            // No subcommand - check for quick callsign lookup
            if !cli.callsigns.is_empty() {
                // Validate all args look like callsigns
                if cli.callsigns.iter().all(|c| looks_like_callsign(c)) {
                    commands::lookup::execute(
                        &cli.callsigns,
                        "auto",
                        cli.all,
                        &cli.format,
                        &staleness_opts,
                    )
                    .await
                } else {
                    // Find first non-callsign to report
                    let bad = cli
                        .callsigns
                        .iter()
                        .find(|c| !looks_like_callsign(c))
                        .unwrap();
                    eprintln!("Unknown command or invalid callsign: {}", bad);
                    eprintln!("Run 'uls --help' for usage information.");
                    std::process::exit(1);
                }
            } else {
                // No command and no callsigns - show help
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
        assert!(!looks_like_callsign("AB")); // too short
        assert!(!looks_like_callsign(""));
        assert!(!looks_like_callsign("12345")); // no letters at start
    }
}
