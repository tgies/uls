//! Query engine for FCC ULS data lookups.
//!
//! This crate provides a high-level API for querying ULS license data.
//! It wraps the database operations and provides convenient search and
//! lookup methods with filtering and formatting support.
//!
//! # Example
//!
//! ```no_run
//! use uls_query::{QueryEngine, SearchFilter};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = QueryEngine::open("~/.uls/uls.db")?;
//!     
//!     // Quick callsign lookup
//!     if let Some(license) = engine.lookup("W1AW")? {
//!         println!("{} - {}", license.call_sign, license.display_name());
//!     }
//!     
//!     // Search by name
//!     let results = engine.search(SearchFilter::name("SMITH"))?;
//!     for license in results {
//!         println!("{}", license.call_sign);
//!     }
//!     
//!     Ok(())
//! }
//! ```

mod engine;
mod fields;
mod filter;
mod output;

pub use engine::QueryEngine;
pub use fields::{FieldDef, FieldRegistry, FieldType, FilterExpr, FilterOp};
pub use filter::{SearchFilter, SortOrder};
pub use output::{FormatOutput, OutputFormat};
pub use uls_db::models::{License, LicenseStats};
