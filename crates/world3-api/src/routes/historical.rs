use std::sync::Arc;

use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
    Json,
};

use crate::{error::ApiError, historical::HistoricalVariable, state::AppState};

/// Cache-Control for static historical data (24 hours).
const CACHE_CONTROL: (header::HeaderName, &str) =
    (header::CACHE_CONTROL, "public, max-age=86400");

// ---------------------------------------------------------------------------
// GET /api/v1/historical
// ---------------------------------------------------------------------------

/// Return all historical variables, sorted by variable id.
pub async fn list_all(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut vars: Vec<HistoricalVariable> = state.historical.values().cloned().collect();
    vars.sort_by(|a, b| a.variable.cmp(&b.variable));
    ([CACHE_CONTROL], Json(vars))
}

// ---------------------------------------------------------------------------
// GET /api/v1/historical/:variable_id
// ---------------------------------------------------------------------------

/// Return a single historical variable by id.
pub async fn get_variable(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(variable_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .historical
        .get(&variable_id)
        .cloned()
        .map(|v| ([CACHE_CONTROL], Json(v)))
        .ok_or_else(|| {
            ApiError::NotFound(format!("Historical variable '{}' not found", variable_id))
        })
}
