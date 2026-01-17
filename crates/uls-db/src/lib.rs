//! SQLite database layer for FCC ULS data storage.
//!
//! This crate provides functionality to store and query FCC ULS data in a
//! SQLite database. It supports both full database builds from weekly files
//! and incremental updates from daily transaction files.
//!
//! # Example
//!
//! ```no_run
//! use uls_db::{Database, DatabaseConfig};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let db = Database::open("uls.db")?;
//!     
//!     // Look up a callsign
//!     if let Some(license) = db.get_license_by_callsign("W1AW")? {
//!         println!("Found: {} - {}", license.call_sign, license.licensee_name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod models;
pub mod repository;
pub mod schema;

pub use config::DatabaseConfig;
pub use error::{DbError, Result};
pub use models::{License, LicenseStats, Operator};
pub use repository::{Database, Transaction};
pub use schema::Schema;
