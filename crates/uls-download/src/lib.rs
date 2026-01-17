//! FCC ULS file download and synchronization.
//!
//! This crate provides functionality to download FCC ULS data files:
//! - Weekly (complete) databases from `https://data.fcc.gov/download/pub/uls/complete/`
//! - Daily transaction files from `https://data.fcc.gov/download/pub/uls/daily/`
//!
//! # Example
//!
//! ```no_run
//! use uls_download::{FccClient, DownloadConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = FccClient::new(DownloadConfig::default())?;
//!     
//!     // Download the amateur radio license database
//!     let path = client.download_complete("amat").await?;
//!     println!("Downloaded to: {}", path.display());
//!     
//!     Ok(())
//! }
//! ```

pub mod catalog;
pub mod client;
pub mod config;
pub mod error;
pub mod progress;

pub use catalog::{DataFile, ServiceCatalog};
pub use client::{FccClient, DownloadResult};
pub use config::DownloadConfig;
pub use error::{DownloadError, Result};
pub use progress::{DownloadProgress, ProgressCallback};
