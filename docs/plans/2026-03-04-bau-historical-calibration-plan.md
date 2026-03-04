# BAU Historical Calibration Regression Test — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a regression test that computes RMSE% between BAU simulation and real-world historical data for 4 variables, with bi-traceability to REQ-026.

**Architecture:** Dedicated integration test file (`tests/historical_calibration.rs`) that depends on `world3-core` directly (since `world3-cli` is binary-only with no `[lib]`). Loads historical CSVs with a minimal inline parser, runs BAU simulation via `Rk4Solver`, extracts time series, computes RMSE% per variable, asserts against thresholds.

**Tech Stack:** Rust, world3-core (Rk4Solver, WorldState, ScenarioParams), CSV parsing (inline, no extra deps)

---

### Task 1: Add world3-core dev-dependency and create test file skeleton

**Files:**
- Modify: `crates/world3-cli/Cargo.toml` — add `world3-core` to `[dev-dependencies]`
- Create: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Add dev-dependency**

In `crates/world3-cli/Cargo.toml`, add under `[dev-dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
world3-core = { path = "../world3-core" }
```

**Step 2: Write the test file skeleton with CSV parser and RMSE helper**

Create `crates/world3-cli/tests/historical_calibration.rs`:

```rust
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
) -> (Vec<f64>, Vec<f64>) {
    let mut sim_vals = Vec::new();
    let mut hist_vals = Vec::new();
    for &(year, hist_val) in historical {
        // Find simulation state at this year (dt=1.0, integer years)
        if let Some(state) = sim.states.iter().find(|s| (s.time - year).abs() < 0.5) {
            sim_vals.push(extract(state));
            hist_vals.push(hist_val);
        }
    }
    (sim_vals, hist_vals)
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

// Placeholder: tests will be added in subsequent tasks.
```

**Step 3: Verify it compiles**

Run: `cargo test -p world3-cli --test historical_calibration --no-run`
Expected: Compiles successfully (no tests to run yet).

**Step 4: Commit**

```bash
git add crates/world3-cli/Cargo.toml crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: scaffold historical calibration test with helpers (REQ-026)"
```

---

### Task 2: Write population test (TDD — failing)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Write the failing test**

Append to `historical_calibration.rs`, replacing the placeholder comment:

```rust
/// REQ-026: BAU population must track World Bank SP.POP.TOTL within 15% RMSE.
#[test]
fn bau_population_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("population.csv"));
    let (sim_vals, hist_vals) = match_years(sim, |s| s.population.population, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 15.0,
        "REQ-026 Population: RMSE% = {:.1}%, threshold = 15.0%",
        pct
    );
}
```

**Step 2: Run test to verify it fails (or passes — either is informative)**

Run: `cargo test -p world3-cli --test historical_calibration bau_population_tracks_historical -- --nocapture`
Expected: Either FAIL with the RMSE% value printed, or PASS if population is well-calibrated. Record the actual RMSE%.

**Step 3: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add population historical calibration test (REQ-026)"
```

---

### Task 3: Write food/capita test (TDD — failing)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Write the failing test**

```rust
/// REQ-026: BAU food/capita must track FAO Food Balance data within 25% RMSE.
#[test]
fn bau_food_per_capita_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("food.csv"));
    let (sim_vals, hist_vals) = match_years(sim, |s| s.agriculture.food_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 25.0,
        "REQ-026 Food/capita: RMSE% = {:.1}%, threshold = 25.0%",
        pct
    );
}
```

**Step 2: Run test**

Run: `cargo test -p world3-cli --test historical_calibration bau_food_per_capita -- --nocapture`
Expected: FAIL with RMSE% printed.

**Step 3: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add food/capita historical calibration test (REQ-026)"
```

---

### Task 4: Write IOPC test (TDD — failing)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Write the failing test**

```rust
/// REQ-026: BAU IOPC must track World Bank industrial VA data within 30% RMSE.
#[test]
fn bau_iopc_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("industrial.csv"));
    let (sim_vals, hist_vals) =
        match_years(sim, |s| s.capital.industrial_output_per_capita, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 30.0,
        "REQ-026 IOPC: RMSE% = {:.1}%, threshold = 30.0%",
        pct
    );
}
```

**Step 2: Run test**

Run: `cargo test -p world3-cli --test historical_calibration bau_iopc -- --nocapture`
Expected: FAIL with RMSE% printed.

**Step 3: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add IOPC historical calibration test (REQ-026)"
```

---

### Task 5: Write NNR fraction test (TDD — failing)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Write the failing test**

```rust
/// REQ-026: BAU NNR fraction must track OWID resource depletion within 20% RMSE.
#[test]
fn bau_nnr_fraction_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("resources.csv"));
    let (sim_vals, hist_vals) = match_years(sim, |s| s.resources.fraction_remaining, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 20.0,
        "REQ-026 NNR: RMSE% = {:.1}%, threshold = 20.0%",
        pct
    );
}
```

**Step 2: Run test**

Run: `cargo test -p world3-cli --test historical_calibration bau_nnr -- --nocapture`
Expected: FAIL with RMSE% printed.

**Step 3: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add NNR fraction historical calibration test (REQ-026)"
```

---

### Task 6: Run all 4 tests, record RMSE% values, add summary test

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Run all tests and record actual RMSE% values**

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture 2>&1`
Expected: Some or all tests FAIL. Record each RMSE% value.

**Step 2: Add a summary test that prints all RMSE% values (always passes)**

Append to the test file:

```rust
/// Summary: print all RMSE% values for visibility. Always passes.
/// REQ-026 traceability — shows current calibration gap.
#[test]
fn calibration_summary_report() {
    let sim = bau_sim();
    let vars: Vec<(&str, &str, fn(&WorldState) -> f64, f64)> = vec![
        ("Population", "population.csv", |s: &WorldState| s.population.population, 15.0),
        ("Food/capita", "food.csv", |s: &WorldState| s.agriculture.food_per_capita, 25.0),
        ("IOPC", "industrial.csv", |s: &WorldState| s.capital.industrial_output_per_capita, 30.0),
        ("NNR fraction", "resources.csv", |s: &WorldState| s.resources.fraction_remaining, 20.0),
    ];
    println!("\n=== BAU Historical Calibration Report (REQ-026) ===");
    for (name, csv, extract, threshold) in vars {
        let hist = load_historical_csv(&historical_dir().join(csv));
        let (sim_vals, hist_vals) = match_years(sim, extract, &hist);
        let pct = rmse_pct(&sim_vals, &hist_vals);
        let status = if pct < threshold { "PASS" } else { "FAIL" };
        println!(
            "  {:<20} RMSE% = {:6.1}%  (threshold: {:5.1}%)  [{}]",
            name, pct, threshold, status
        );
    }
    println!("========================================================\n");
}
```

**Step 3: Run full suite**

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture 2>&1`
Expected: Summary prints, individual tests show PASS/FAIL.

**Step 4: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add calibration summary report and record RMSE% baseline (REQ-026)"
```

---

### Task 7: Update CLAUDE.md with REQ-026 documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add calibration requirement to CLAUDE.md**

Under the "Validation Baseline" section, add:

```markdown
### Historical Calibration (REQ-026)
- BAU simulation must track real-world historical data within RMSE% thresholds over ~1960-2023.
- Variables: Population (<15%), Food/capita (<25%), IOPC (<30%), NNR fraction (<20%).
- Test: `cargo test -p world3-cli --test historical_calibration`
- Design: `docs/plans/2026-03-04-bau-historical-calibration-design.md`
- Currently FAILING — thresholds are aspirational calibration targets.
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add REQ-026 historical calibration requirement to CLAUDE.md"
```

---

### Task 8: Final verification

**Step 1: Run full test suite to ensure nothing is broken**

Run: `cargo test --workspace`
Expected: All existing tests pass. Historical calibration tests may fail (expected).

Run: `cargo clippy -p world3-cli -- -D warnings`
Expected: No warnings.

**Step 2: Verify bi-traceability**

Check that:
- Design doc references REQ-026 and lists all 4 test functions
- Test file header documents REQ-026 and traceability matrix
- CLAUDE.md references REQ-026, test command, and design doc
- Each `#[test]` has a doc comment citing REQ-026

**Step 3: Final commit if any adjustments needed**

```bash
git add -A && git commit -m "chore: final adjustments for REQ-026 calibration tests"
```
