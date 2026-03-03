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
/// Values calibrated to broadly match Meadows 1972 standard run starting point.
pub fn initial_conditions_1900() -> WorldState {
    WorldState {
        time: 1900.0,
        population: PopulationState {
            population: 1.6e9,
            // World3-03 initial cohorts (Meadows 2004)
            cohort_0_14: 6.5e8,
            cohort_15_44: 7.0e8,
            cohort_45_64: 1.9e8,
            cohort_65_plus: 6.0e7,
            perceived_le: 33.0,  // Initial perceived LE matches 1900 computed LE
            ..Default::default()
        },
        capital: CapitalState {
            industrial_capital: 2.1e11,  // World3-03: ici = 2.1e11
            service_capital: 1.44e11,    // World3-03: sci = 1.44e11
            // 1900 IOPC ≈ IC/ICOR/POP = 2.1e11 / 3.0 / 1.6e9 ≈ 43.75
            perceived_iopc: 43.75,
            ..Default::default()
        },
        agriculture: AgricultureState {
            arable_land: 0.9e9,
            potentially_arable_land: 2.3e9,
            urban_industrial_land: 8.2e6,  // World3-03: uili = 8.2e6 hectares
            land_fertility: 600.0,         // World3-03: lferti = 600 kg/ha/yr
            food_per_capita: 400.0,
            ..Default::default()
        },
        resources: ResourceState {
            nonrenewable_resources: 1.0,
            fraction_remaining: 1.0,
        },
        pollution: PollutionState {
            persistent_pollution: 0.05,
            pollution_appearance_buffer: 0.05 * 20.0,
            pollution_index: 0.05,
            ..Default::default()
        },
    }
}
