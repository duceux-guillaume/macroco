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
        "collapse" => Ok(ScenarioParams::collapse()),
        "technology" => Ok(ScenarioParams::technotopia()),
        "stabilized" => Ok(ScenarioParams::ecotopia()),
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

/// Convergence threshold: max allowed relative drift between dt halvings.
/// 3% accommodates sharp transient peaks (e.g. pollution spike) that are
/// inherently dt-sensitive in system-dynamics models while still catching
/// genuine numerical instability.
const STABILITY_THRESHOLD: f64 = 0.03; // 3%

/// Run the simulation at multiple dt values and report convergence.
///
/// Uses dt, dt/2, and dt/4. A variable is considered converged when the
/// relative change in both final value and peak value between consecutive
/// halvings stays below `STABILITY_THRESHOLD`.
pub fn run_stability_check(
    preset_name: &str,
    start: f64,
    end: f64,
    dt: f64,
) -> Result<analysis::StabilityReport> {
    let dt_values = vec![dt, dt / 2.0, dt / 4.0];
    let diags: Vec<SimDiagnostics> = dt_values
        .iter()
        .map(|&d| run_analysis(preset_name, start, end, d))
        .collect::<Result<Vec<_>>>()?;

    let var_count = diags[0].variables.len();
    let mut all_converged = true;
    let mut variables = Vec::with_capacity(var_count);

    for vi in 0..var_count {
        let final_values: Vec<f64> = diags.iter().map(|d| d.variables[vi].final_value).collect();
        let peak_values: Vec<f64> = diags.iter().map(|d| d.variables[vi].peak.value).collect();
        let phase_counts: Vec<usize> = diags.iter().map(|d| d.variables[vi].phases.len()).collect();

        let max_final_drift = consecutive_max_relative_change(&final_values);
        let max_peak_drift = consecutive_max_relative_change(&peak_values);

        let converged =
            max_final_drift < STABILITY_THRESHOLD && max_peak_drift < STABILITY_THRESHOLD;
        if !converged {
            all_converged = false;
        }

        variables.push(analysis::VariableStability {
            name: diags[0].variables[vi].name.clone(),
            final_values,
            peak_values,
            phase_counts,
            max_final_value_drift: max_final_drift,
            max_peak_drift,
            converged,
        });
    }

    Ok(analysis::StabilityReport {
        preset_name: preset_name.to_string(),
        dt_values,
        variables,
        stable: all_converged,
    })
}

/// Max relative change between consecutive elements in a slice.
fn consecutive_max_relative_change(vals: &[f64]) -> f64 {
    vals.windows(2)
        .map(|w| {
            let base = w[0].abs().max(w[1].abs());
            if base < 1e-12 {
                0.0
            } else {
                (w[1] - w[0]).abs() / base
            }
        })
        .fold(0.0_f64, f64::max)
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
        let var_diag = analysis::analyze_variable(tv.name, tv.unit, &years, &values);
        let oscillations = analysis::detect_oscillations(tv.name, &var_diag.phases);
        all_anomalies.extend(oscillations);
        variables.push(var_diag);
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

// REQ: REQ-003
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_diagnostics_regression() {
        let diag = run_analysis("collapse", 1900.0, 2100.0, 1.0).expect("Collapse sim failed");

        // Population peaks in expected range
        let pop = &diag.variables[0];
        assert_eq!(pop.name, "Population");
        assert!(pop.peak.year >= 2000.0 && pop.peak.year <= 2090.0,
            "Population peak year {} outside [2000, 2090]", pop.peak.year);
        assert!(pop.peak.value >= 5.0e9 && pop.peak.value <= 16.0e9,
            "Population peak value {:.2e} outside [5B, 16B]", pop.peak.value);
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

        // No anomalies in standard Collapse run
        assert!(diag.anomalies.is_empty(),
            "Collapse should have no anomalies, found: {:?}", diag.anomalies);
    }

    #[test]
    fn comparative_collapse_vs_technology() {
        let base = run_analysis("collapse", 1900.0, 2100.0, 1.0).expect("Collapse failed");
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
        let diag = run_analysis("collapse", 1900.0, 2100.0, 1.0).expect("Collapse failed");
        let json = format_json::format_json(&diag);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(parsed["variables"].as_array().unwrap().len(), 6);
        assert!(parsed["anomalies"].as_array().unwrap().is_empty());
    }

    #[test]
    fn technology_no_oscillation_after_ifpc_fix() {
        // After the IFPC food allocation rework, Technology preset no longer oscillates at dt=1.0
        let diag = run_analysis("technology", 1900.0, 2100.0, 1.0).expect("Tech sim failed");
        let oscillations: Vec<_> = diag.anomalies.iter()
            .filter(|a| a.kind == analysis::AnomalyKind::Oscillation)
            .collect();
        assert!(oscillations.is_empty(),
            "Technology preset should not have oscillation anomalies after IFPC fix, found: {:?}",
            oscillations.iter().map(|a| &a.variable).collect::<Vec<_>>());
    }

    #[test]
    fn collapse_stability_check_passes() {
        let report = run_stability_check("collapse", 1900.0, 2100.0, 1.0).expect("Collapse stability failed");
        assert!(report.stable, "Collapse should be stable at dt=1.0");
        assert_eq!(report.dt_values.len(), 3);
        for vs in &report.variables {
            assert!(vs.converged, "{} should converge in Collapse", vs.name);
        }
    }

    #[test]
    fn stability_json_roundtrips() {
        let report = run_stability_check("collapse", 1900.0, 2100.0, 1.0).expect("Collapse stability failed");
        let json = format_json::format_json_stability(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(parsed["stable"], true);
        assert_eq!(parsed["variables"].as_array().unwrap().len(), 6);
    }
}
