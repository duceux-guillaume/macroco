use serde::{Deserialize, Serialize};
use world3_core::{
    model::state::{
        AgricultureState, CapitalState, PollutionState, PopulationState, ResourceState, WorldState,
    },
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

#[derive(Debug, Deserialize)]
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
        state: WorldState,
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
/// Values calibrated to broadly match Meadows 1972 standard run starting point.
pub fn initial_conditions_1900() -> WorldState {
    WorldState {
        time: 1900.0,
        population: PopulationState {
            population: 1.6e9,
            cohort_0_14: 0.60e9,
            cohort_15_44: 0.65e9,
            cohort_45_64: 0.27e9,
            cohort_65_plus: 0.08e9,
            ..Default::default()
        },
        capital: CapitalState {
            industrial_capital: 0.2e12,
            service_capital: 0.32e12,
            ..Default::default()
        },
        agriculture: AgricultureState {
            arable_land: 0.9e9,
            potentially_arable_land: 2.3e9,
            food_per_capita: 400.0,
            ..Default::default()
        },
        resources: ResourceState {
            nonrenewable_resources: 1.0,
            fraction_remaining: 1.0,
        },
        pollution: PollutionState {
            persistent_pollution: 0.05,
            pollution_index: 0.05,
            ..Default::default()
        },
    }
}
