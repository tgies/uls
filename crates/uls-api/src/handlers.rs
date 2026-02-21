//! Request handlers for the API endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uls_query::{QueryEngine, SearchFilter};

use crate::error::ApiError;
use crate::response::ListResponse;

/// Shared application state.
pub type AppState = Arc<QueryEngine>;

/// GET /health
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// GET /stats
pub async fn stats(State(engine): State<AppState>) -> Result<Json<Value>, ApiError> {
    let stats = engine.stats()?;
    Ok(Json(serde_json::to_value(stats).map_err(|e| {
        ApiError::Internal(e.to_string())
    })?))
}

/// GET /licenses/:callsign
pub async fn lookup(
    State(engine): State<AppState>,
    Path(callsign): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let license = engine
        .lookup(&callsign)?
        .ok_or_else(|| ApiError::NotFound(format!("No license found for callsign {}", callsign)))?;

    Ok(Json(
        serde_json::to_value(license).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

/// GET /frn/:frn
pub async fn frn_lookup(
    State(engine): State<AppState>,
    Path(frn): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if frn.len() != 10 || !frn.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest(
            "FRN must be exactly 10 digits".to_string(),
        ));
    }

    let licenses = engine.lookup_by_frn(&frn)?;
    if licenses.is_empty() {
        return Err(ApiError::NotFound(format!(
            "No licenses found for FRN {}",
            frn
        )));
    }

    let response = ListResponse::new(licenses, 0, 0);
    Ok(Json(
        serde_json::to_value(response).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

/// Query parameters for the search endpoint.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub name: Option<String>,
    pub callsign: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub zip: Option<String>,
    pub frn: Option<String>,
    pub status: Option<String>,
    pub class: Option<String>,
    pub service: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub active: Option<bool>,
    pub granted_after: Option<String>,
    pub granted_before: Option<String>,
    pub expires_before: Option<String>,
    pub filter: Option<String>,
}

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 1000;

/// GET /licenses
pub async fn search(
    State(engine): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, ApiError> {
    let mut filter = SearchFilter::new();

    if let Some(ref name) = params.name {
        let pattern = if name.contains('*') || name.contains('?') {
            name.clone()
        } else {
            format!("*{}*", name)
        };
        filter.name = Some(pattern);
    }

    if let Some(ref callsign) = params.callsign {
        filter.callsign = Some(callsign.clone());
    }

    if let Some(ref state) = params.state {
        filter.state = Some(state.clone());
    }

    if let Some(ref city) = params.city {
        filter = filter.with_filter(format!("city={}", city));
    }

    if let Some(ref zip) = params.zip {
        filter.zip_code = Some(zip.clone());
    }

    if let Some(ref frn) = params.frn {
        filter.frn = Some(frn.clone());
    }

    if let Some(ref status) = params.status {
        if let Some(c) = status.chars().next() {
            filter.status = Some(c.to_ascii_uppercase());
        }
    }

    if let Some(ref class) = params.class {
        if let Some(c) = class.chars().next() {
            filter = filter.with_operator_class(c.to_ascii_uppercase());
        }
    }

    if let Some(ref service) = params.service {
        let codes = match service.to_lowercase().as_str() {
            "amateur" | "ham" | "ha" => vec!["HA".to_string(), "HV".to_string()],
            "gmrs" | "za" => vec!["ZA".to_string()],
            _ => return Err(ApiError::BadRequest(format!("Unknown service: {}", service))),
        };
        filter.radio_service = Some(codes);
    }

    if params.active == Some(true) {
        filter = filter.active_only();
    }

    filter.granted_after = params.granted_after;
    filter.granted_before = params.granted_before;
    filter.expires_before = params.expires_before;

    if let Some(ref filter_str) = params.filter {
        for expr in filter_str.split(',') {
            let trimmed = expr.trim();
            if !trimmed.is_empty() {
                filter = filter.with_filter(trimmed);
            }
        }
    }

    if let Some(ref sort) = params.sort {
        filter = filter.with_sort_field(sort);
    }

    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0);
    filter = filter.with_limit(limit);
    if offset > 0 {
        filter = filter.with_offset(offset);
    }

    let licenses = engine.search(filter)?;
    let response = ListResponse::new(licenses, limit, offset);

    Ok(Json(
        serde_json::to_value(response).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}
