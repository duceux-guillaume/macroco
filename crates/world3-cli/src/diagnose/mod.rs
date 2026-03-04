pub mod analysis;
pub mod compare;
pub mod format_json;
pub mod format_text;

use anyhow::Result;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

pub use analysis::SimDiagnostics;

struct TrackedVar {
    name: &'static str,
    unit: &'static str,
    extract: fn(&WorldState) -> f64,
}

fn tracked_variables() -> Vec<TrackedVar> {
    vec![
        TrackedVar {
            name: "Population",
            unit: "people",
            extract: |s| s.population.population,
        },
        TrackedVar {
            name: "Food / capita",
            unit: "kg/person/yr",
            extract: |s| s.agriculture.food_per_capita,
        },
        TrackedVar {
            name: "Industrial output / capita",
            unit: "USD/person/yr",
            extract: |s| s.capital.industrial_output_per_capita,
        },
        TrackedVar {
            name: "Services / capita",
            unit: "USD/person/yr",
            extract: |s| s.capital.service_output_per_capita,
        },
        TrackedVar {
            name: "NNR fraction",
            unit: "fraction",
            extract: |s| s.resources.fraction_remaining,
        },
        TrackedVar {
            name: "Pollution index",
            unit: "index (1970=1)",
            extract: |s| s.pollution.pollution_index,
        },
    ]
}

pub fn preset_params(name: &str) -> Result<ScenarioParams> {
    match name {
        "bau" => Ok(ScenarioParams::bau()),
        "technology" => Ok(ScenarioParams::comprehensive_technology()),
        "stabilized" => Ok(ScenarioParams::stabilized_world()),
        other => anyhow::bail!("Unknown preset '{}'", other),
    }
}

pub fn run_sim(preset_name: &str, start: f64, end: f64, dt: f64) -> Result<SimulationOutput> {
    let mut params = preset_params(preset_name)?;
    params.start_year = start;
    params.end_year = end;
    params.time_step = dt;
    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    Ok(SimulationOutput::new(states, params))
}

pub fn run_analysis(preset_name: &str, start: f64, end: f64, dt: f64) -> Result<SimDiagnostics> {
    let sim = run_sim(preset_name, start, end, dt)?;
    Ok(analyze_sim(&sim, preset_name))
}

pub fn analyze_sim(sim: &SimulationOutput, preset_name: &str) -> SimDiagnostics {
    let years: Vec<f64> = sim.states.iter().map(|s| s.time).collect();
    let vars = tracked_variables();
    let mut all_anomalies = Vec::new();
    let mut variables = Vec::new();

    for tv in &vars {
        let values: Vec<f64> = sim.states.iter().map(|s| (tv.extract)(s)).collect();
        let anomalies = analysis::detect_anomalies(tv.name, &years, &values);
        all_anomalies.extend(anomalies);
        variables.push(analysis::analyze_variable(tv.name, tv.unit, &years, &values));
    }

    SimDiagnostics {
        preset_name: preset_name.to_string(),
        time_range: (*years.first().unwrap_or(&0.0), *years.last().unwrap_or(&0.0)),
        dt: sim.params.time_step,
        num_steps: sim.states.len(),
        variables,
        anomalies: all_anomalies,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bau_diagnostics_regression() {
        let diag = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU sim failed");

        // Population peaks in expected range
        let pop = &diag.variables[0];
        assert_eq!(pop.name, "Population");
        assert!(pop.peak.year >= 2000.0 && pop.peak.year <= 2070.0,
            "Population peak year {} outside [2000, 2070]", pop.peak.year);
        assert!(pop.peak.value >= 5.0e9 && pop.peak.value <= 12.0e9,
            "Population peak value {:.2e} outside [5B, 12B]", pop.peak.value);
        assert!(!pop.is_monotonic, "Population should not be monotonic");
        assert!(pop.phases.len() >= 2, "Population should have at least 2 phases");

        // NNR is monotonically declining
        let nnr = &diag.variables[4];
        assert_eq!(nnr.name, "NNR fraction");
        assert!(nnr.is_monotonic, "NNR should be monotonically declining");

        // Food per capita should peak then decline
        let food = &diag.variables[1];
        assert_eq!(food.name, "Food / capita");
        assert!(food.peak.value > food.final_value, "Food/cap should peak then decline");

        // IOPC should collapse
        let iopc = &diag.variables[2];
        assert!(iopc.final_value < iopc.peak.value * 0.5,
            "IOPC should collapse by 2100");

        // No anomalies in standard BAU run
        assert!(diag.anomalies.is_empty(),
            "BAU should have no anomalies, found: {:?}", diag.anomalies);
    }

    #[test]
    fn comparative_bau_vs_technology() {
        let base = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU failed");
        let comp = run_analysis("technology", 1900.0, 2100.0, 1.0).expect("Tech failed");
        let result = compare::compare(base, comp);

        assert_eq!(result.deltas.len(), 6, "Should have 6 variable deltas");

        // All deltas computed without NaN
        for d in &result.deltas {
            assert!(!d.peak_value_change.is_nan(), "{} peak_value_change is NaN", d.name);
            assert!(!d.peak_value_pct_change.is_nan(), "{} pct_change is NaN", d.name);
            assert!(!d.peak_year_shift.is_nan(), "{} peak_year_shift is NaN", d.name);
            assert!(!d.final_value_change.is_nan(), "{} final_value_change is NaN", d.name);
        }
    }

    #[test]
    fn json_output_roundtrips() {
        let diag = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU failed");
        let json = format_json::format_json(&diag);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(parsed["variables"].as_array().unwrap().len(), 6);
        assert!(parsed["anomalies"].as_array().unwrap().is_empty());
    }
}
