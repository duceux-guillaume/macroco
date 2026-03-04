// REQ: REQ-026
//! BAU Historical Calibration Regression Tests
//!
//! Requirement: REQ-026
//! The BAU simulation output shall remain within acceptable RMSE% thresholds
//! of real-world historical data for the overlapping time period (~1960-2023).
//!
//! Design: docs/plans/2026-03-04-bau-historical-calibration-design.md
//! Traceability matrix:
//!   REQ-026 (Population)   -> bau_population_tracks_historical
//!   REQ-026 (Food/capita)  -> bau_food_per_capita_tracks_historical
//!   REQ-026 (IOPC)         -> bau_iopc_tracks_historical
//!   REQ-026 (NNR fraction) -> bau_nnr_fraction_tracks_historical

use std::path::Path;
use std::sync::OnceLock;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

// ---------------------------------------------------------------------------
// Shared BAU simulation (run once across all tests)
// ---------------------------------------------------------------------------

fn bau_sim() -> &'static SimulationOutput {
    static SIM: OnceLock<SimulationOutput> = OnceLock::new();
    SIM.get_or_init(|| {
        let params = ScenarioParams::bau();
        let initial = WorldState::initial_1900();
        let tables = std::sync::Arc::new(
            world3_core::lookup::tables::WorldLookupTables::load(),
        );
        let solver = Rk4Solver::new(tables);
        let states = solver.solve(initial, &params).expect("BAU simulation failed");
        SimulationOutput::new(states, params)
    })
}

// ---------------------------------------------------------------------------
// CSV loader (minimal — handles comment-header format from data/historical/)
// ---------------------------------------------------------------------------

fn load_historical_csv(path: &Path) -> Vec<(f64, f64)> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with("year"))
        .filter_map(|line| {
            let mut parts = line.split(',');
            let year: f64 = parts.next()?.trim().parse().ok()?;
            let value: f64 = parts.next()?.trim().parse().ok()?;
            if year.is_finite() && value.is_finite() {
                Some((year, value))
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// RMSE% computation
// ---------------------------------------------------------------------------

/// Extract simulation values at years matching historical data.
/// Returns (matched_sim_values, matched_hist_values).
fn match_years(
    sim: &SimulationOutput,
    extract: fn(&WorldState) -> f64,
    historical: &[(f64, f64)],
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut sim_vals = Vec::new();
    let mut hist_vals = Vec::new();
    let mut years = Vec::new();
    for &(year, hist_val) in historical {
        // Find simulation state at this year (dt=1.0, integer years)
        if let Some(state) = sim.states.iter().find(|s| (s.time - year).abs() < 0.5) {
            sim_vals.push(extract(state));
            hist_vals.push(hist_val);
            years.push(year);
        }
    }
    (sim_vals, hist_vals, years)
}

/// Maximum absolute percentage error across all matched years.
/// Returns (max_error_pct, worst_year).
fn max_year_error_pct(sim_vals: &[f64], hist_vals: &[f64], hist_years: &[f64]) -> (f64, f64) {
    assert_eq!(sim_vals.len(), hist_vals.len());
    assert_eq!(sim_vals.len(), hist_years.len());
    assert!(!sim_vals.is_empty(), "No overlapping years found");
    let mut max_err = 0.0_f64;
    let mut worst_year = 0.0_f64;
    for i in 0..sim_vals.len() {
        let err = ((sim_vals[i] - hist_vals[i]) / hist_vals[i]).abs() * 100.0;
        if err > max_err {
            max_err = err;
            worst_year = hist_years[i];
        }
    }
    (max_err, worst_year)
}

/// RMSE as percentage of mean historical value.
fn rmse_pct(sim_vals: &[f64], hist_vals: &[f64]) -> f64 {
    assert_eq!(sim_vals.len(), hist_vals.len());
    assert!(!sim_vals.is_empty(), "No overlapping years found");
    let n = sim_vals.len() as f64;
    let mse: f64 = sim_vals
        .iter()
        .zip(hist_vals)
        .map(|(s, h)| (s - h).powi(2))
        .sum::<f64>()
        / n;
    let rmse = mse.sqrt();
    let mean_hist = hist_vals.iter().sum::<f64>() / n;
    (rmse / mean_hist) * 100.0
}

// ---------------------------------------------------------------------------
// Data directory
// ---------------------------------------------------------------------------

fn historical_dir() -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../data/historical")
}

// ---------------------------------------------------------------------------
// Tests — REQ-026
// ---------------------------------------------------------------------------

/// REQ-026: BAU population must track World Bank SP.POP.TOTL within 15% RMSE.
/// Current RMSE% = 14.1% after pyworld3 alignment.
#[test]
fn bau_population_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.population.population, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 15.0,
        "REQ-026 Population: RMSE% = {:.1}%, threshold = 15.0%",
        pct
    );
}

/// REQ-026: BAU food/capita must track FAO Food Balance data within 25% RMSE.
/// Current RMSE% = 24.8% after pyworld3 alignment.
#[test]
fn bau_food_per_capita_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 25.0,
        "REQ-026 Food/capita: RMSE% = {:.1}%, threshold = 25.0%",
        pct
    );
}

/// REQ-026: BAU IOPC must track World Bank industrial VA data within 30% RMSE.
/// Current RMSE% = 28.2% after pyworld3 alignment.
#[test]
fn bau_iopc_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 30.0,
        "REQ-026 IOPC: RMSE% = {:.1}%, threshold = 30.0%",
        pct
    );
}

/// REQ-026: BAU NNR fraction must track OWID resource depletion within 20% RMSE.
/// Current RMSE% = 4.3% after pyworld3 alignment.
#[test]
fn bau_nnr_fraction_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 20.0,
        "REQ-026 NNR: RMSE% = {:.1}%, threshold = 20.0%",
        pct
    );
}

/// REQ-026: BAU population max per-year error must be ≤ 30%.
#[test]
fn bau_population_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.population.population, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 30.0,
        "REQ-026 Population max-year: {:.1}% in year {} (threshold 30.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU food/capita max per-year error must be ≤ 30%.
#[test]
fn bau_food_per_capita_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 30.0,
        "REQ-026 Food/capita max-year: {:.1}% in year {} (threshold 30.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU IOPC max per-year error must be ≤ 30%.
#[test]
fn bau_iopc_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 30.0,
        "REQ-026 IOPC max-year: {:.1}% in year {} (threshold 30.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU NNR fraction max per-year error must be ≤ 30%.
#[test]
fn bau_nnr_fraction_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 30.0,
        "REQ-026 NNR max-year: {:.1}% in year {} (threshold 30.0%)",
        max_err, worst_year
    );
}

/// Summary: print all RMSE% values for visibility. Always passes.
/// REQ-026 traceability — shows current calibration gap.
#[test]
fn calibration_summary_report() {
    let sim = bau_sim();
    let vars: Vec<(&str, &str, fn(&WorldState) -> f64, f64)> = vec![
        ("Population", "population.csv", (|s: &WorldState| s.population.population) as fn(&WorldState) -> f64, 15.0),
        ("Food/capita", "food.csv", (|s: &WorldState| s.agriculture.food_per_capita) as fn(&WorldState) -> f64, 25.0),
        ("IOPC", "industrial.csv", (|s: &WorldState| s.capital.industrial_output_per_capita) as fn(&WorldState) -> f64, 30.0),
        ("NNR fraction", "resources.csv", (|s: &WorldState| s.resources.fraction_remaining) as fn(&WorldState) -> f64, 20.0),
    ];
    println!("\n=== BAU Historical Calibration Report (REQ-026) ===");
    for (name, csv, extract, threshold) in vars {
        let hist = load_historical_csv(&historical_dir().join(csv));
        let (sim_vals, hist_vals, years) = match_years(sim, extract, &hist);
        let pct = rmse_pct(&sim_vals, &hist_vals);
        let status = if pct < threshold { "PASS" } else { "FAIL" };
        let (max_err, worst_yr) = max_year_error_pct(&sim_vals, &hist_vals, &years);
        let max_status = if max_err <= 30.0 { "PASS" } else { "FAIL" };
        println!(
            "  {:<20} RMSE% = {:6.1}%  (threshold: {:5.1}%)  [{}]  max-year: {:5.1}% @ {} [{}]",
            name, pct, threshold, status, max_err, worst_yr, max_status
        );
    }
    println!("========================================================\n");
}
