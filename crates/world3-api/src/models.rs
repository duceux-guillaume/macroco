use serde::{Deserialize, Serialize};
use world3_core::{
    model::state::WorldState,
    output::SimulationOutput,
    ScenarioParams,
};

// ---------------------------------------------------------------------------
// Scenario
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub params: ScenarioParams,
    pub is_preset: bool,
    pub last_output: Option<SimulationOutput>,
}

/// Lightweight summary returned in list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color_hex: String,
    pub is_preset: bool,
}

impl From<&Scenario> for ScenarioSummary {
    fn from(s: &Scenario) -> Self {
        ScenarioSummary {
            id: s.params.meta.id.clone(),
            name: s.params.meta.name.clone(),
            description: s.params.meta.description.clone(),
            color_hex: s.params.meta.color_hex.clone(),
            is_preset: s.is_preset,
        }
    }
}

// ---------------------------------------------------------------------------
// WebSocket messages
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMsg {
    StartSimulation {
        scenario_id: String,
        params: Option<ScenarioParams>,
    },
    UpdateParams {
        scenario_id: String,
        params: ScenarioParams,
    },
    StopSimulation,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMsg {
    SimStep {
        year: f64,
        state: Box<WorldState>,
    },
    SimComplete {
        scenario_id: String,
        total_steps: usize,
    },
    SimError {
        message: String,
    },
    ParamsAck {
        scenario_id: String,
    },
}

// ---------------------------------------------------------------------------
// Initial conditions for 1900
// ---------------------------------------------------------------------------

/// World 3 initial conditions for year 1900.
/// Delegates to `WorldState::initial_1900()` in world3-core.
pub fn initial_conditions_1900() -> WorldState {
    WorldState::initial_1900()
}
