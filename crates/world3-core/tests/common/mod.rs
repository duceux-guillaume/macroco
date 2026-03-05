use std::path::Path;
use std::sync::OnceLock;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

/// Run a simulation with the given params. Used inside OnceLock initializers.
pub fn run_sim(params: ScenarioParams) -> SimulationOutput {
    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let name = params.meta.name.clone();
    let states = solver
        .solve(initial, &params)
        .unwrap_or_else(|_| panic!("{name} simulation failed"));
    SimulationOutput::new(states, params)
}

/// Shared Collapse simulation (run once across all tests in this binary).
pub fn collapse_sim() -> &'static SimulationOutput {
    static SIM: OnceLock<SimulationOutput> = OnceLock::new();
    SIM.get_or_init(|| run_sim(ScenarioParams::collapse()))
}

/// Shared Technotopia simulation (run once across all tests in this binary).
pub fn technotopia_sim() -> &'static SimulationOutput {
    static SIM: OnceLock<SimulationOutput> = OnceLock::new();
    SIM.get_or_init(|| run_sim(ScenarioParams::technotopia()))
}

// ---------------------------------------------------------------------------
// Shared calibration helpers
// ---------------------------------------------------------------------------

/// Path to bundled historical CSV data.
pub fn historical_dir() -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../data/historical")
}

/// Load a historical CSV file (comment-header format from data/historical/).
pub fn load_historical_csv(path: &Path) -> Vec<(f64, f64)> {
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

/// Extract simulation values at years matching historical data.
/// Returns (matched_sim_values, matched_hist_values, matched_years).
pub fn match_years(
    sim: &SimulationOutput,
    extract: fn(&WorldState) -> f64,
    historical: &[(f64, f64)],
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut sim_vals = Vec::new();
    let mut hist_vals = Vec::new();
    let mut years = Vec::new();
    for &(year, hist_val) in historical {
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
pub fn max_year_error_pct(sim_vals: &[f64], hist_vals: &[f64], hist_years: &[f64]) -> (f64, f64) {
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
pub fn rmse_pct(sim_vals: &[f64], hist_vals: &[f64]) -> f64 {
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
