use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use world3_core::{output::SimulationOutput, solver::traits::OdeSolver, ScenarioParams};

use crate::{
    error::ApiError,
    models::{initial_conditions_1900, Scenario, ScenarioSummary},
    state::AppState,
};

// ---------------------------------------------------------------------------
// GET /api/v1/scenarios
// ---------------------------------------------------------------------------

pub async fn list_scenarios(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ScenarioSummary>>, ApiError> {
    let store = state.scenarios.read().await;
    let summaries: Vec<ScenarioSummary> = store.values().map(ScenarioSummary::from).collect();
    Ok(Json(summaries))
}

// ---------------------------------------------------------------------------
// GET /api/v1/presets
// ---------------------------------------------------------------------------

pub async fn list_presets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ScenarioSummary>>, ApiError> {
    let store = state.scenarios.read().await;
    let summaries: Vec<ScenarioSummary> = store
        .values()
        .filter(|s| s.is_preset)
        .map(ScenarioSummary::from)
        .collect();
    Ok(Json(summaries))
}

// ---------------------------------------------------------------------------
// POST /api/v1/scenarios
// ---------------------------------------------------------------------------

pub async fn create_scenario(
    State(state): State<Arc<AppState>>,
    Json(params): Json<ScenarioParams>,
) -> Result<Json<Scenario>, ApiError> {
    let scenario = Scenario {
        params: params.clone(),
        is_preset: false,
        last_output: None,
    };
    let id = params.meta.id.clone();
    let mut store = state.scenarios.write().await;
    store.insert(id, scenario.clone());
    Ok(Json(scenario))
}

// ---------------------------------------------------------------------------
// GET /api/v1/scenarios/:id
// ---------------------------------------------------------------------------

pub async fn get_scenario(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Scenario>, ApiError> {
    let store = state.scenarios.read().await;
    let scenario = store
        .get(&id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Scenario '{}' not found", id)))?;
    Ok(Json(scenario))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/scenarios/:id/params
// ---------------------------------------------------------------------------

pub async fn update_params(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(params): Json<ScenarioParams>,
) -> Result<Json<Scenario>, ApiError> {
    let mut store = state.scenarios.write().await;
    let scenario = store
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Scenario '{}' not found", id)))?;
    scenario.params = params;
    scenario.last_output = None;
    Ok(Json(scenario.clone()))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/scenarios/:id
// ---------------------------------------------------------------------------

pub async fn delete_scenario(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut store = state.scenarios.write().await;
    let scenario = store
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Scenario '{}' not found", id)))?;
    if scenario.is_preset {
        return Err(ApiError::Forbidden("Cannot delete preset scenarios".into()));
    }
    store.remove(&id);
    Ok(Json(serde_json::json!({ "deleted": id })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/scenarios/:id/run
// ---------------------------------------------------------------------------

pub async fn run_scenario(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SimulationOutput>, ApiError> {
    // Read params from store
    let params = {
        let store = state.scenarios.read().await;
        store
            .get(&id)
            .map(|s| s.params.clone())
            .ok_or_else(|| ApiError::NotFound(format!("Scenario '{}' not found", id)))?
    };

    // Run simulation on blocking thread pool
    let solver = Arc::clone(&state.solver);
    let initial = initial_conditions_1900();
    let result = tokio::task::spawn_blocking(move || solver.solve(initial, &params))
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Task panicked: {}", e)))?;

    let states = result.map_err(|e| ApiError::SimulationFailed(e.to_string()))?;

    // Build output
    let output = {
        let store = state.scenarios.read().await;
        let scenario_params = store.get(&id).map(|s| s.params.clone()).ok_or_else(|| {
            ApiError::NotFound(format!("Scenario '{}' not found after run", id))
        })?;
        SimulationOutput::new(states, scenario_params)
    };

    // Store last_output
    {
        let mut store = state.scenarios.write().await;
        if let Some(scenario) = store.get_mut(&id) {
            scenario.last_output = Some(output.clone());
        }
    }

    Ok(Json(output))
}
