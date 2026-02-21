//! Server configuration and router construction.

use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uls_query::QueryEngine;

use crate::handlers::{self, AppState};

/// Configuration for the API server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (e.g., "127.0.0.1").
    pub bind: String,
    /// Listen port.
    pub port: u16,
    /// Allowed CORS origins (empty = no CORS layer).
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".to_string(),
            port: 3000,
            cors_origins: Vec::new(),
        }
    }
}

/// Build the axum router with all routes and middleware.
pub fn build_router(engine: QueryEngine, config: &ServerConfig) -> Router {
    let state: AppState = Arc::new(engine);

    let mut app = Router::new()
        .route("/health", axum::routing::get(handlers::health))
        .route("/stats", axum::routing::get(handlers::stats))
        .route("/licenses/{callsign}", axum::routing::get(handlers::lookup))
        .route("/licenses", axum::routing::get(handlers::search))
        .route("/frn/{frn}", axum::routing::get(handlers::frn_lookup))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    if !config.cors_origins.is_empty() {
        let cors = if config.cors_origins.iter().any(|o| o == "*") {
            CorsLayer::new().allow_origin(Any)
        } else {
            let origins: Vec<_> = config
                .cors_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();
            CorsLayer::new().allow_origin(origins)
        };
        app = app.layer(cors);
    }

    app
}

/// Start the server with the given configuration.
pub async fn run(engine: QueryEngine, config: ServerConfig) -> std::io::Result<()> {
    let app = build_router(engine, &config);
    let addr = format!("{}:{}", config.bind, config.port);

    tracing::info!("Starting ULS API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .await
        .expect("server should not fail");
    Ok(())
}
