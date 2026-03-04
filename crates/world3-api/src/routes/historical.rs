use std::sync::Arc;

use axum::{extract::State, Json};

use crate::{error::ApiError, historical::HistoricalVariable, state::AppState};

// ---------------------------------------------------------------------------
// GET /api/v1/historical
// ---------------------------------------------------------------------------

/// Return all historical variables, sorted by variable id.
pub async fn list_all(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<HistoricalVariable>> {
    let mut vars: Vec<HistoricalVariable> = state.historical.values().cloned().collect();
    vars.sort_by(|a, b| a.variable.cmp(&b.variable));
    Json(vars)
}

// ---------------------------------------------------------------------------
// GET /api/v1/historical/:variable_id
// ---------------------------------------------------------------------------

/// Return a single historical variable by id.
pub async fn get_variable(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(variable_id): axum::extract::Path<String>,
) -> Result<Json<HistoricalVariable>, ApiError> {
    state
        .historical
        .get(&variable_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            ApiError::NotFound(format!("Historical variable '{}' not found", variable_id))
        })
}
