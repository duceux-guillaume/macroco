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
        "pollution.pollution_appearance_buffer" => Some(state.pollution.pollution_appearance_buffer),
        "pollution.pollution_index" => Some(state.pollution.pollution_index),
        "agriculture.urban_industrial_land" => Some(state.agriculture.urban_industrial_land),
        "agriculture.land_fertility" => Some(state.agriculture.land_fertility),
        "population.perceived_le" => Some(state.population.perceived_le),
        "hwi" => Some(state.hwi()),
        "ecological_footprint" => Some(state.ecological_footprint()),
        _ => None,
    }
}

// REQ: REQ-003
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::state::WorldState;

    fn make_test_output() -> SimulationOutput {
        let mut states = Vec::new();
        for year in [1900.0, 1950.0, 2000.0, 2050.0, 2100.0] {
            let mut s = WorldState::initial_1900();
            s.time = year;
            s.population.population = 1.6e9 + (year - 1900.0) * 1e7;
            states.push(s);
        }
        let params = ScenarioParams::collapse();
        SimulationOutput::new(states, params)
    }

    #[test]
    fn test_state_at_year_exact() {
        let sim = make_test_output();
        let s = sim.state_at_year(1950.0).unwrap();
        assert_eq!(s.time, 1950.0);
    }

    #[test]
    fn test_state_at_year_interpolation() {
        let sim = make_test_output();
        // 1975 is between 1950 and 2000 — should return closest
        let s = sim.state_at_year(1975.0).unwrap();
        assert!(s.time == 1950.0 || s.time == 2000.0);
        // Distance to 1975 should be <= 25
        assert!((s.time - 1975.0).abs() <= 25.0);
    }

    #[test]
    fn test_extract_series_known_path() {
        let sim = make_test_output();
        let series = sim.extract_series("population.population");
        assert_eq!(series.len(), 5);
        assert!(!series[0].is_nan());
        // First value should be ~1.6e9
        assert!((series[0] - 1.6e9).abs() < 1e6);
    }

    #[test]
    fn test_extract_series_unknown_path() {
        let sim = make_test_output();
        let series = sim.extract_series("nonexistent.field");
        assert_eq!(series.len(), 5);
        for val in &series {
            assert!(val.is_nan());
        }
    }
}
