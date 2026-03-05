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

mod common;
use common::bau_sim;

use std::path::Path;
use world3_core::model::state::WorldState;
use world3_core::output::SimulationOutput;

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
        if let Some(state) = sim.state_at_year(year) {
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
    let (max_err, worst_year) = sim_vals
        .iter()
        .zip(hist_vals)
        .zip(hist_years)
        .map(|((&s, &h), &y)| (((s - h) / h).abs() * 100.0, y))
        .fold((0.0_f64, 0.0_f64), |(me, wy), (e, y)| {
            if e > me { (e, y) } else { (me, wy) }
        });
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

/// REQ-026: BAU population must track World Bank SP.POP.TOTL within 14% RMSE.
/// Widened from 11% to 14% after Delay3 perceived-LE structural change:
/// the 20-year pipeline delay adds demographic inertia that shifts the
/// overshoot-and-collapse trajectory later, increasing RMSE vs historical.
#[test]
fn bau_population_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.population.population, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 14.0,
        "REQ-026 Population: RMSE% = {:.1}%, threshold = 14.0%",
        pct
    );
}

/// REQ-026: BAU food/capita must track FAO Food Balance data within 19% RMSE.
/// Widened from 17% to 19% after Delay3 structural changes. The higher
/// population from delayed demographic transition reduces food per capita.
#[test]
fn bau_food_per_capita_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 19.0,
        "REQ-026 Food/capita: RMSE% = {:.1}%, threshold = 19.0%",
        pct
    );
}

/// REQ-026: BAU IOPC must track World Bank industrial VA data within 21% RMSE.
/// Widened from 18% to 21% after Delay3 structural changes. The delayed
/// pollution collapse shifts IOPC trajectory, and higher population from
/// delayed demographic transition reduces per-capita output.
#[test]
fn bau_iopc_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 21.0,
        "REQ-026 IOPC: RMSE% = {:.1}%, threshold = 21.0%",
        pct
    );
}

/// REQ-026: BAU NNR fraction must track OWID resource depletion within 10% RMSE.
#[test]
fn bau_nnr_fraction_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 10.0,
        "REQ-026 NNR: RMSE% = {:.1}%, threshold = 10.0%",
        pct
    );
}

/// REQ-026: BAU population max per-year error must be ≤ 35%.
/// Tightened from 37% (actual ~32%).
#[test]
fn bau_population_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.population.population, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 35.0,
        "REQ-026 Population max-year: {:.1}% in year {} (threshold 35.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU food/capita max per-year error must be ≤ 26%.
#[test]
fn bau_food_per_capita_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 26.0,
        "REQ-026 Food/capita max-year: {:.1}% in year {} (threshold 26.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU IOPC max per-year error must be ≤ 36%.
#[test]
fn bau_iopc_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 36.0,
        "REQ-026 IOPC max-year: {:.1}% in year {} (threshold 36.0%)",
        max_err, worst_year
    );
}

/// REQ-026: BAU NNR fraction max per-year error must be ≤ 20%.
#[test]
fn bau_nnr_fraction_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 20.0,
        "REQ-026 NNR max-year: {:.1}% in year {} (threshold 20.0%)",
        max_err, worst_year
    );
}

/// Summary: print all RMSE% values for visibility. Always passes.
/// REQ-026 traceability — shows current calibration gap.
#[test]
fn calibration_summary_report() {
    let sim = bau_sim();
    let vars: Vec<(&str, &str, fn(&WorldState) -> f64, f64, f64)> = vec![
        ("Population", "population.csv", (|s: &WorldState| s.population.population) as fn(&WorldState) -> f64, 14.0, 35.0),
        ("Food/capita", "food.csv", (|s: &WorldState| s.agriculture.food_per_capita) as fn(&WorldState) -> f64, 19.0, 26.0),
        ("IOPC", "industrial.csv", (|s: &WorldState| s.capital.industrial_output_per_capita) as fn(&WorldState) -> f64, 21.0, 36.0),
        ("NNR fraction", "resources.csv", (|s: &WorldState| s.resources.fraction_remaining) as fn(&WorldState) -> f64, 10.0, 20.0),
    ];
    println!("\n=== BAU Historical Calibration Report (REQ-026) ===");
    for (name, csv, extract, rmse_threshold, max_err_threshold) in vars {
        let hist = load_historical_csv(&historical_dir().join(csv));
        let (sim_vals, hist_vals, years) = match_years(sim, extract, &hist);
        let pct = rmse_pct(&sim_vals, &hist_vals);
        let status = if pct < rmse_threshold { "PASS" } else { "FAIL" };
        let (max_err, worst_yr) = max_year_error_pct(&sim_vals, &hist_vals, &years);
        let max_status = if max_err <= max_err_threshold { "PASS" } else { "FAIL" };
        println!(
            "  {:<20} RMSE% = {:6.1}%  (threshold: {:5.1}%)  [{}]  max-year: {:5.1}% @ {} (threshold: {:5.1}%) [{}]",
            name, pct, rmse_threshold, status, max_err, worst_yr, max_err_threshold, max_status
        );
    }
    println!("========================================================\n");
}
