//! Serve command - start the REST API server.

use anyhow::{Context, Result};
use uls_api::ServerConfig;
use uls_query::QueryEngine;

use crate::config::default_db_path;

pub async fn execute(
    port: u16,
    bind: &str,
    cors_origins: Vec<String>,
) -> Result<()> {
    let db_path = default_db_path();

    let engine = QueryEngine::open(&db_path)
        .context("Failed to open database. Run 'uls update' first to initialize.")?;

    let config = ServerConfig {
        bind: bind.to_string(),
        port,
        cors_origins,
    };

    eprintln!(
        "ULS API server listening on http://{}:{}",
        config.bind, config.port
    );

    uls_api::run(engine, config)
        .await
        .context("Server error")?;

    Ok(())
}
