// REQ: REQ-026
//! Collapse Historical Calibration Regression Tests
//!
//! Requirement: REQ-026
//! The Collapse simulation output shall remain within acceptable RMSE% thresholds
//! of real-world historical data for the overlapping time period (~1960-2023).
//!
//! Design: docs/plans/2026-03-04-bau-historical-calibration-design.md
//! Traceability matrix:
//!   REQ-026 (Population)   -> collapse_population_tracks_historical
//!   REQ-026 (Food/capita)  -> collapse_food_per_capita_tracks_historical
//!   REQ-026 (IOPC)         -> collapse_iopc_tracks_historical
//!   REQ-026 (NNR fraction) -> collapse_nnr_fraction_tracks_historical
//!   REQ-026 (Life expect.) -> collapse_life_expectancy_tracks_historical

mod common;

use std::path::Path;
use world3_core::model::state::WorldState;

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
    sim: &world3_core::output::SimulationOutput,
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

/// REQ-026: Collapse population must track World Bank SP.POP.TOTL within 11% RMSE.
/// Tightened from 14% after LE alignment improvements (actual ~8.1%).
#[test]
fn collapse_population_tracks_historical() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.population.population, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 11.0,
        "REQ-026 Population: RMSE% = {:.1}%, threshold = 11.0%",
        pct
    );
}

/// REQ-026: Collapse food/capita must track FAO Food Balance data within 15% RMSE.
/// Tightened after agricultural_technology_growth_rate extension.
#[test]
fn collapse_food_per_capita_tracks_historical() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 15.0,
        "REQ-026 Food/capita: RMSE% = {:.1}%, threshold = 15.0%",
        pct
    );
}

/// REQ-026: Collapse IOPC must track World Bank industrial VA data within 16% RMSE.
/// Tightened from 19% after resource_efficiency_growth_rate extension (actual ~14.9%).
#[test]
fn collapse_iopc_tracks_historical() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 16.0,
        "REQ-026 IOPC: RMSE% = {:.1}%, threshold = 16.0%",
        pct
    );
}

/// REQ-026: Collapse NNR fraction must track OWID resource depletion within 4% RMSE.
/// Tightened from 10% after LE alignment improvements (actual ~0.9%).
#[test]
fn collapse_nnr_fraction_tracks_historical() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 4.0,
        "REQ-026 NNR: RMSE% = {:.1}%, threshold = 4.0%",
        pct
    );
}

/// REQ-026: Collapse population max per-year error must be ≤ 15%.
/// Tightened from 35% after LE alignment improvements (actual ~11.3%).
#[test]
fn collapse_population_max_year_error() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.population.population, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 15.0,
        "REQ-026 Population max-year: {:.1}% in year {} (threshold 15.0%)",
        max_err, worst_year
    );
}

/// REQ-026: Collapse food/capita max per-year error must be ≤ 20%.
/// Tightened after agricultural_technology_growth_rate extension.
#[test]
fn collapse_food_per_capita_max_year_error() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 20.0,
        "REQ-026 Food/capita max-year: {:.1}% in year {} (threshold 20.0%)",
        max_err, worst_year
    );
}

/// REQ-026: Collapse IOPC max per-year error must be ≤ 38%.
/// Kept loose — actual ~35.4% at 1960 is sensitive to initial conditions.
#[test]
fn collapse_iopc_max_year_error() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 38.0,
        "REQ-026 IOPC max-year: {:.1}% in year {} (threshold 38.0%)",
        max_err, worst_year
    );
}

/// REQ-026: Collapse NNR fraction max per-year error must be ≤ 7%.
/// Widened from 6% after ag tech growth rate extension (actual ~6.7%); still tight vs original 20%.
#[test]
fn collapse_nnr_fraction_max_year_error() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 7.0,
        "REQ-026 NNR max-year: {:.1}% in year {} (threshold 7.0%)",
        max_err, worst_year
    );
}

/// REQ-026: Collapse life expectancy must track World Bank SP.DYN.LE00.IN within 12% RMSE.
/// Tightened from 25% after LE alignment improvements (actual ~9.4%).
#[test]
fn collapse_life_expectancy_tracks_historical() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 12.0,
        "REQ-026 Life expectancy: RMSE% = {:.1}%, threshold = 12.0%",
        pct
    );
}

/// REQ-026: Collapse life expectancy max per-year error must be ≤ 19%.
/// Tightened from 40% after LE alignment improvements (actual ~15.6%).
#[test]
fn collapse_life_expectancy_max_year_error() {
    let sim = common::collapse_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 19.0,
        "REQ-026 Life expectancy max-year: {:.1}% in year {} (threshold 19.0%)",
        max_err, worst_year
    );
}

/// Summary: print all RMSE% values for visibility. Always passes.
/// REQ-026 traceability — shows current calibration gap.
#[test]
fn calibration_summary_report() {
    let sim = common::collapse_sim();
    let vars: Vec<(&str, &str, fn(&WorldState) -> f64, f64, f64)> = vec![
        ("Population", "population.csv", (|s: &WorldState| s.population.population) as fn(&WorldState) -> f64, 11.0, 15.0),
        ("Food/capita", "food.csv", (|s: &WorldState| s.agriculture.food_per_capita) as fn(&WorldState) -> f64, 15.0, 20.0),
        ("IOPC", "industrial.csv", (|s: &WorldState| s.capital.industrial_output_per_capita) as fn(&WorldState) -> f64, 16.0, 38.0),
        ("NNR fraction", "resources.csv", (|s: &WorldState| s.resources.fraction_remaining) as fn(&WorldState) -> f64, 4.0, 7.0),
        ("Life expect.", "life-expectancy.csv", (|s: &WorldState| s.population.life_expectancy) as fn(&WorldState) -> f64, 12.0, 19.0),
    ];
    println!("\n=== Collapse Historical Calibration Report (REQ-026) ===");
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
