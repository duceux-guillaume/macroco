use serde::{Deserialize, Serialize};

use crate::model::{params::ScenarioParams, state::WorldState};

/// A complete simulation run: the time series of all world states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationOutput {
    pub scenario_id: String,
    pub scenario_name: String,
    /// Simulation years corresponding to each state
    pub timeline: Vec<f64>,
    /// One WorldState per time step, same length as timeline
    pub states: Vec<WorldState>,
    /// Parameters used for this run
    pub params: ScenarioParams,
    /// ISO-8601 timestamp when the simulation was computed
    pub computed_at: String,
}

impl SimulationOutput {
    pub fn new(states: Vec<WorldState>, params: ScenarioParams) -> Self {
        let timeline = states.iter().map(|s| s.time).collect();
        Self {
            scenario_id: params.meta.id.clone(),
            scenario_name: params.meta.name.clone(),
            timeline,
            states,
            params,
            computed_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Return the state closest to the requested year.
    pub fn state_at_year(&self, year: f64) -> Option<&WorldState> {
        self.states
            .iter()
            .min_by(|a, b| {
                (a.time - year)
                    .abs()
                    .partial_cmp(&(b.time - year).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Extract a single named variable as a time series.
    /// Supported paths: "population.population", "agriculture.food_per_capita", etc.
    pub fn extract_series(&self, path: &str) -> Vec<f64> {
        self.states
            .iter()
            .map(|s| extract_field(s, path).unwrap_or(f64::NAN))
            .collect()
    }
}

fn extract_field(state: &WorldState, path: &str) -> Option<f64> {
    match path {
        "population.population" => Some(state.population.population),
        "population.birth_rate" => Some(state.population.birth_rate),
        "population.death_rate" => Some(state.population.death_rate),
        "population.life_expectancy" => Some(state.population.life_expectancy),
        "population.fertility_rate" => Some(state.population.fertility_rate),
        "capital.industrial_capital" => Some(state.capital.industrial_capital),
        "capital.service_capital" => Some(state.capital.service_capital),
        "capital.industrial_output" => Some(state.capital.industrial_output),
        "capital.industrial_output_per_capita" => Some(state.capital.industrial_output_per_capita),
        "capital.service_output_per_capita" => Some(state.capital.service_output_per_capita),
        "agriculture.arable_land" => Some(state.agriculture.arable_land),
        "agriculture.food" => Some(state.agriculture.food),
        "agriculture.food_per_capita" => Some(state.agriculture.food_per_capita),
        "agriculture.land_yield" => Some(state.agriculture.land_yield),
        "resources.nonrenewable_resources" => Some(state.resources.nonrenewable_resources),
        "resources.fraction_remaining" => Some(state.resources.fraction_remaining),
        "pollution.persistent_pollution" => Some(state.pollution.persistent_pollution),
        "pollution.pollution_index" => Some(state.pollution.pollution_index),
        _ => None,
    }
}
