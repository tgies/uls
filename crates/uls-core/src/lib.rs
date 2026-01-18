//! Core data types and models for FCC ULS (Universal Licensing System) data.
//!
//! This crate provides:
//! - Data structures for all 89 ULS record types
//! - Enums for radio service codes and various status codes
//! - Common traits and utilities for working with ULS data

pub mod codes;
pub mod error;
pub mod records;

pub use codes::*;
pub use error::{Error, Result};
pub use records::*;
