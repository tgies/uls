//! REST API server for FCC ULS data access.

pub mod error;
pub mod handlers;
pub mod response;
pub mod server;

pub use server::{build_router, run, ServerConfig};
