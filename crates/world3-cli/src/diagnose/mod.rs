#[allow(dead_code)]
pub mod analysis;
#[allow(dead_code)]
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
pub use compare::ComparativeDiagnostics;

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
