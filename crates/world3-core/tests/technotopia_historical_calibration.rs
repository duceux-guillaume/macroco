// REQ: REQ-035
//! Technotopia Historical Calibration Regression Tests
//!
//! Requirement: REQ-035
//! The Technotopia simulation output shall remain within acceptable RMSE% thresholds
//! of real-world historical data for the overlapping time period (~1960-2023).
//! The historical fit should be comparable to Collapse — differences are model noise.
//!
//! Design: docs/plans/2026-03-05-technotopia-calibration-design.md

mod common;

use common::{historical_dir, load_historical_csv, match_years, max_year_error_pct, rmse_pct};
use world3_core::model::state::WorldState;

// ---------------------------------------------------------------------------
// Tests — REQ-035 Historical Calibration
// ---------------------------------------------------------------------------

/// REQ-035: Technotopia population must track historical data within RMSE threshold.
/// Slightly relaxed vs Collapse (13% vs 11%) — param differences create small divergence.
#[test]
fn technotopia_population_tracks_historical() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.population.population, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 13.0,
        "REQ-035 Population: RMSE% = {:.1}%, threshold = 13.0%",
        pct
    );
}

/// REQ-035: Technotopia food/capita must track historical data within RMSE threshold.
#[test]
fn technotopia_food_per_capita_tracks_historical() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 18.0,
        "REQ-035 Food/capita: RMSE% = {:.1}%, threshold = 18.0%",
        pct
    );
}

/// REQ-035: Technotopia IOPC must track historical data within RMSE threshold.
#[test]
fn technotopia_iopc_tracks_historical() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 22.0,
        "REQ-035 IOPC: RMSE% = {:.1}%, threshold = 22.0%",
        pct
    );
}

/// REQ-035: Technotopia NNR fraction must track historical data within RMSE threshold.
#[test]
fn technotopia_nnr_fraction_tracks_historical() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, _years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 6.0,
        "REQ-035 NNR: RMSE% = {:.1}%, threshold = 6.0%",
        pct
    );
}

/// REQ-035: Technotopia life expectancy must track historical data within RMSE threshold.
#[test]
fn technotopia_life_expectancy_tracks_historical() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 14.0,
        "REQ-035 Life expectancy: RMSE% = {:.1}%, threshold = 14.0%",
        pct
    );
}

/// REQ-035: Technotopia population max per-year error.
#[test]
fn technotopia_population_max_year_error() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.population.population, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 18.0,
        "REQ-035 Population max-year: {:.1}% in year {} (threshold 18.0%)",
        max_err, worst_year
    );
}

/// REQ-035: Technotopia food/capita max per-year error.
#[test]
fn technotopia_food_per_capita_max_year_error() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 25.0,
        "REQ-035 Food/capita max-year: {:.1}% in year {} (threshold 25.0%)",
        max_err, worst_year
    );
}

/// REQ-035: Technotopia IOPC max per-year error.
#[test]
fn technotopia_iopc_max_year_error() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 42.0,
        "REQ-035 IOPC max-year: {:.1}% in year {} (threshold 42.0%)",
        max_err, worst_year
    );
}

/// REQ-035: Technotopia NNR fraction max per-year error.
#[test]
fn technotopia_nnr_fraction_max_year_error() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals, years) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 15.0,
        "REQ-035 NNR max-year: {:.1}% in year {} (threshold 15.0%)",
        max_err, worst_year
    );
}

/// REQ-035: Technotopia life expectancy max per-year error.
#[test]
fn technotopia_life_expectancy_max_year_error() {
    let sim = common::technotopia_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 22.0,
        "REQ-035 Life expectancy max-year: {:.1}% in year {} (threshold 22.0%)",
        max_err, worst_year
    );
}

// ---------------------------------------------------------------------------
// Qualitative dynamics — Technotopia must NOT collapse
// ---------------------------------------------------------------------------

/// REQ-035: Technotopia population should stabilize, not peak-and-crash like Collapse.
/// Population at 2100 must be > 6 billion (no collapse).
#[test]
fn technotopia_population_does_not_collapse() {
    let sim = common::technotopia_sim();
    let state_2100 = sim.state_at_year(2100.0).expect("No state at 2100");
    let pop_2100 = state_2100.population.population;
    assert!(
        pop_2100 > 6.0e9,
        "Technotopia population at 2100 = {:.2e}, expected > 6e9 (no collapse)",
        pop_2100
    );
}

/// REQ-035: Technotopia IOPC should remain elevated (no industrial collapse).
/// Collapse drops to ~49 by 2100; Technotopia should stay well above that.
#[test]
fn technotopia_iopc_stays_elevated() {
    let sim = common::technotopia_sim();
    let min_iopc_post_2050 = (2050..=2100)
        .filter_map(|y| sim.state_at_year(y as f64))
        .map(|s| s.capital.industrial_output_per_capita)
        .fold(f64::INFINITY, f64::min);
    assert!(
        min_iopc_post_2050 > 120.0,
        "Technotopia min IOPC post-2050 = {:.1}, expected > 120 (no deep collapse)",
        min_iopc_post_2050
    );
}

/// REQ-035: Technotopia food/capita should not crash below subsistence.
#[test]
fn technotopia_food_above_subsistence() {
    let sim = common::technotopia_sim();
    let state_2100 = sim.state_at_year(2100.0).expect("No state at 2100");
    let food_2100 = state_2100.agriculture.food_per_capita;
    assert!(
        food_2100 > 230.0,
        "Technotopia food/capita at 2100 = {:.1}, expected > 230 (above subsistence)",
        food_2100
    );
}

// ---------------------------------------------------------------------------
// Summary report
// ---------------------------------------------------------------------------

/// Summary: print all RMSE% values for visibility. Always passes.
#[test]
fn technotopia_calibration_summary_report() {
    let sim = common::technotopia_sim();
    let vars: Vec<(&str, &str, fn(&WorldState) -> f64, f64, f64)> = vec![
        ("Population", "population.csv", (|s: &WorldState| s.population.population) as fn(&WorldState) -> f64, 13.0, 18.0),
        ("Food/capita", "food.csv", (|s: &WorldState| s.agriculture.food_per_capita) as fn(&WorldState) -> f64, 18.0, 25.0),
        ("IOPC", "industrial.csv", (|s: &WorldState| s.capital.industrial_output_per_capita) as fn(&WorldState) -> f64, 22.0, 42.0),
        ("NNR fraction", "resources.csv", (|s: &WorldState| s.resources.fraction_remaining) as fn(&WorldState) -> f64, 6.0, 15.0),
        ("Life expect.", "life-expectancy.csv", (|s: &WorldState| s.population.life_expectancy) as fn(&WorldState) -> f64, 14.0, 22.0),
    ];
    println!("\n=== Technotopia Historical Calibration Report (REQ-035) ===");
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
    println!("==========================================================\n");
}
