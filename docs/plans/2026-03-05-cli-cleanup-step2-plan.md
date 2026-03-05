# CLI Cleanup Step 2: Validation Module + Test Migration

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract validation logic into `world3-core::validation`, move all scenario tests from `world3-cli` to `world3-core`, and make CLI `validate` a thin wrapper.

**Architecture:** Create a `validation` module in `world3-core` with a `validate_bau()` function that returns structured `CheckResult` values. Move `historical_calibration.rs` to `world3-core/tests/` with a shared `bau_sim()` helper. Replace the ~200-line inline `validate()` in `main.rs` with a 20-line wrapper that calls the module and prints results.

**Tech Stack:** Rust, `world3-core` crate, `cargo test`

**Design:** `docs/plans/2026-03-05-cli-cleanup-design.md` (Step 2 section)

---

### Task 1: Create `world3-core::validation` module with `CheckResult` type

**Files:**
- Create: `crates/world3-core/src/validation.rs`
- Modify: `crates/world3-core/src/lib.rs:1`

**Step 1: Create the module with types and `validate_bau()` function**

Create `crates/world3-core/src/validation.rs`:

```rust
//! BAU validation against World3 reference dynamics (Meadows 1972/2004).
//!
//! Checks qualitative dynamics (not exact values) of the BAU standard run:
//! population overshoot-and-collapse, resource depletion, pollution peak,
//! industrial collapse, life expectancy decline.

use crate::output::SimulationOutput;

/// Result of a single validation check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Human-readable label (e.g. "Population peak")
    pub label: String,
    /// Whether the check passed
    pub passed: bool,
    /// Detail string (e.g. "8.2e9 at year 2035, expected [5B-12B, 1990-2080]")
    pub detail: String,
}

/// Run all BAU qualitative validation checks against a simulation output.
///
/// Returns a `Vec<CheckResult>` — one per check. The caller decides how to
/// present results (CLI prints PASS/FAIL, tests assert all passed).
///
/// Checks:
/// 1. Population at 1900, 1950, 1970 within expected ranges
/// 2. Population peaks 5B-12B between 1990-2080
/// 3. Population declines after peak
/// 4. NNR fraction at 2000 and 2100 within ranges
/// 5. NNR monotonically decreasing
/// 6. Pollution peak within range
/// 7. IOPC peaks then collapses before 2100
/// 8. Life expectancy peaks 45-80yr then declines
/// 9. Life expectancy at 2100 below 80% of peak
pub fn validate_bau(sim: &SimulationOutput) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // --- Population trajectory ---
    for (year, lo, hi) in [(1900.0, 1.4e9, 1.8e9), (1950.0, 2.0e9, 4.0e9), (1970.0, 3.0e9, 5.5e9)] {
        let pop = sim.state_at_year(year).map(|s| s.population.population).unwrap_or(f64::NAN);
        let passed = (lo..=hi).contains(&pop);
        results.push(CheckResult {
            label: format!("{:.0} population", year),
            passed,
            detail: format!("{:.3e} [expected {:.1e}–{:.1e}]", pop, lo, hi),
        });
    }

    // Population peak
    let (peak_pop, peak_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.population > mp { (s.population.population, s.time) } else { (mp, my) }
        });
    let peak_ok = (5.0e9..=12.0e9).contains(&peak_pop) && (1990.0..=2080.0).contains(&peak_year);
    results.push(CheckResult {
        label: "Population peak".into(),
        passed: peak_ok,
        detail: format!("{:.2e} at year {:.0} [expected 5B–12B, 1990–2080]", peak_pop, peak_year),
    });

    // Population decline after peak
    let pop_2100 = sim.state_at_year(2100.0).map(|s| s.population.population).unwrap_or(f64::NAN);
    let decline_ok = pop_2100 < peak_pop * 0.95;
    results.push(CheckResult {
        label: "Population decline after peak".into(),
        passed: decline_ok,
        detail: format!("{:.2e} at 2100 vs peak {:.2e}", pop_2100, peak_pop),
    });

    // --- Resource depletion ---
    for (year, lo, hi) in [(2000.0, 0.0, 0.60), (2100.0, 0.0, 0.30)] {
        let nnr = sim.state_at_year(year).map(|s| s.resources.fraction_remaining).unwrap_or(f64::NAN);
        let passed = (lo..=hi).contains(&nnr);
        results.push(CheckResult {
            label: format!("{:.0} NNR fraction", year),
            passed,
            detail: format!("{:.4} [expected {:.2}–{:.2}]", nnr, lo, hi),
        });
    }

    // NNR monotonic
    let checkpoints = [1920.0, 1940.0, 1960.0, 1980.0, 2000.0, 2020.0, 2040.0, 2060.0, 2080.0, 2100.0];
    let nnr_monotonic = checkpoints.windows(2).all(|pair| {
        let a = sim.state_at_year(pair[0]).map(|s| s.resources.fraction_remaining).unwrap_or(1.0);
        let b = sim.state_at_year(pair[1]).map(|s| s.resources.fraction_remaining).unwrap_or(0.0);
        b <= a + 0.001
    });
    results.push(CheckResult {
        label: "NNR monotonically decreasing".into(),
        passed: nnr_monotonic,
        detail: if nnr_monotonic { "OK".into() } else { "Non-monotonic NNR detected".into() },
    });

    // --- Pollution ---
    let max_pollution = sim.states.iter()
        .map(|s| s.pollution.pollution_index)
        .fold(0.0_f64, f64::max);
    let poll_ok = (1.0..=100.0).contains(&max_pollution);
    results.push(CheckResult {
        label: "Peak pollution index".into(),
        passed: poll_ok,
        detail: format!("{:.2} [expected 1.0–100.0]", max_pollution),
    });

    // --- IOPC collapse ---
    let (peak_iopc, peak_iopc_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.capital.industrial_output_per_capita > mp {
                (s.capital.industrial_output_per_capita, s.time)
            } else { (mp, my) }
        });
    let iopc_2100 = sim.state_at_year(2100.0)
        .map(|s| s.capital.industrial_output_per_capita).unwrap_or(f64::NAN);
    let iopc_ok = iopc_2100 <= peak_iopc * 0.5;
    results.push(CheckResult {
        label: "IOPC collapse".into(),
        passed: iopc_ok,
        detail: format!("{:.0} at 2100, peak {:.0} at {:.0} [expected <50% of peak]", iopc_2100, peak_iopc, peak_iopc_year),
    });

    // --- Life expectancy ---
    let (peak_le, peak_le_year) = sim.states.iter()
        .filter(|s| s.time >= 1910.0)
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.life_expectancy > mp {
                (s.population.life_expectancy, s.time)
            } else { (mp, my) }
        });
    let le_peak_ok = (45.0..=80.0).contains(&peak_le);
    results.push(CheckResult {
        label: "Peak life expectancy".into(),
        passed: le_peak_ok,
        detail: format!("{:.1} yr at {:.0} [expected 45–80]", peak_le, peak_le_year),
    });

    let le_2100 = sim.state_at_year(2100.0)
        .map(|s| s.population.life_expectancy).unwrap_or(f64::NAN);
    let le_decline_ok = le_2100 <= peak_le * 0.8;
    results.push(CheckResult {
        label: "Life expectancy decline".into(),
        passed: le_decline_ok,
        detail: format!("{:.1} at 2100 vs peak {:.1} at {:.0}", le_2100, peak_le, peak_le_year),
    });

    results
}
```

**Step 2: Register the module in `lib.rs`**

In `crates/world3-core/src/lib.rs`, add `pub mod validation;` after `pub mod solver;`.

**Step 3: Verify it compiles**

Run: `cargo build -p world3-core`
Expected: compiles with no errors.

**Step 4: Commit**

```bash
git add crates/world3-core/src/validation.rs crates/world3-core/src/lib.rs
git commit -m "feat: add world3-core::validation module with validate_bau()"
```

---

### Task 2: Create shared `bau_sim()` helper in `world3-core/tests/common/`

**Files:**
- Create: `crates/world3-core/tests/common/mod.rs`

**Step 1: Create the shared helper**

Create `crates/world3-core/tests/common/mod.rs`:

```rust
use std::sync::OnceLock;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

pub fn bau_sim() -> &'static SimulationOutput {
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
```

**Step 2: Verify it compiles**

Run: `cargo test -p world3-core --no-run`
Expected: compiles (no tests use it yet, but the module must parse).

**Step 3: Commit**

```bash
git add crates/world3-core/tests/common/mod.rs
git commit -m "feat: add shared bau_sim() test helper in world3-core"
```

---

### Task 3: Create `qualitative_dynamics.rs` integration test in `world3-core`

**Files:**
- Create: `crates/world3-core/tests/qualitative_dynamics.rs`

**Step 1: Write the test**

Create `crates/world3-core/tests/qualitative_dynamics.rs`:

```rust
// REQ: REQ-001
//! BAU Qualitative Dynamics Tests
//!
//! Validates that the BAU simulation reproduces World3 overshoot-and-collapse
//! dynamics via the world3_core::validation module.

mod common;

use world3_core::validation::validate_bau;

#[test]
fn bau_all_qualitative_checks_pass() {
    let sim = common::bau_sim();
    let results = validate_bau(sim);

    let mut failures = Vec::new();
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        eprintln!("  {}  {}: {}", status, r.label, r.detail);
        if !r.passed {
            failures.push(format!("{}: {}", r.label, r.detail));
        }
    }

    assert!(
        failures.is_empty(),
        "BAU qualitative validation failed:\n{}",
        failures.join("\n")
    );
}
```

**Step 2: Run the test**

Run: `cargo test -p world3-core --test qualitative_dynamics -- --nocapture`
Expected: PASS — all checks pass (the same logic that was inline in CLI `validate()`).

**Step 3: Commit**

```bash
git add crates/world3-core/tests/qualitative_dynamics.rs
git commit -m "test: add qualitative dynamics integration test in world3-core"
```

---

### Task 4: Move `historical_calibration.rs` from `world3-cli` to `world3-core`

**Files:**
- Create: `crates/world3-core/tests/historical_calibration.rs` (moved from `world3-cli`)
- Delete: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Copy and adapt the test file**

Copy `crates/world3-cli/tests/historical_calibration.rs` to `crates/world3-core/tests/historical_calibration.rs`.

Changes needed:
1. Replace the inline `bau_sim()` function with `mod common;` and `use common::bau_sim;` (shared helper from Task 2).
2. Remove the duplicated `bau_sim()` function (lines 28-40 of the original).
3. Everything else stays identical — `CARGO_MANIFEST_DIR` + `../../data/historical` resolves correctly from `crates/world3-core/`.

The `use world3_core::...` imports remain unchanged since this is an integration test in `world3-core` (integration tests use the crate as an external dependency).

**Step 2: Run the moved tests**

Run: `cargo test -p world3-core --test historical_calibration -- --nocapture`
Expected: All 9 tests pass (8 threshold tests + 1 summary report).

**Step 3: Delete the original file**

Delete `crates/world3-cli/tests/historical_calibration.rs`.

**Step 4: Verify world3-cli tests still pass**

Run: `cargo test -p world3-cli`
Expected: passes (no integration tests remain, only the binary compiles).

**Step 5: Commit**

```bash
git add crates/world3-core/tests/historical_calibration.rs
git rm crates/world3-cli/tests/historical_calibration.rs
git commit -m "refactor: move historical calibration tests to world3-core"
```

---

### Task 5: Replace CLI inline `validate()` with thin wrapper

**Files:**
- Modify: `crates/world3-cli/src/main.rs:120-465`

**Step 1: Replace the `validate()` function**

In `crates/world3-cli/src/main.rs`, replace the entire `validate()` function (lines 262-465) with:

```rust
fn validate() -> Result<()> {
    eprintln!("Running BAU validation against World3 reference dynamics…\n");

    let params = ScenarioParams::bau();
    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    let sim = SimulationOutput::new(states, params);

    let results = world3_core::validation::validate_bau(&sim);

    let mut failed = false;
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        eprintln!("  {}  {}: {}", status, r.label, r.detail);
        if !r.passed {
            failed = true;
        }
    }

    eprintln!();
    if failed {
        anyhow::bail!("Validation failed");
    } else {
        eprintln!("Validation PASSED ({} checks)", results.len());
        Ok(())
    }
}
```

**Step 2: Verify the CLI works**

Run: `cargo run --bin world3-cli -- validate`
Expected: prints PASS for all checks, exits 0.

**Step 3: Run full workspace tests**

Run: `cargo test --workspace`
Expected: all pass.

**Step 4: Commit**

```bash
git add crates/world3-cli/src/main.rs
git commit -m "refactor: replace inline validate() with world3-core::validation wrapper"
```

---

### Task 6: Update documentation

**Files:**
- Modify: `docs/architecture.md:21-103`
- Modify: `CLAUDE.md`

**Step 1: Update `docs/architecture.md`**

In the `## Simulation Engine (world3-core)` section (after line 31), add:

```markdown
### Validation Module

- `world3_core::validation::validate_bau()` runs qualitative dynamics checks against a `SimulationOutput` and returns `Vec<CheckResult>`.
- Used by: `world3-core/tests/qualitative_dynamics.rs` (integration tests), CLI `validate` command (thin wrapper).
- Checks: population trajectory, NNR depletion, pollution peak, IOPC collapse, life expectancy decline.
```

In the `## CLI (world3-cli)` section, update the historical calibration subsection:

Replace:
```
### Historical Calibration Tests (REQ-026)

- Integration test in `crates/world3-cli/tests/historical_calibration.rs`
```

With:
```
### Historical Calibration Tests (REQ-026)

- Integration test in `crates/world3-core/tests/historical_calibration.rs`
```

**Step 2: Update `CLAUDE.md`**

In the Backend Testing section, update references:
- Change `cargo test -p world3-cli --test historical_calibration` → `cargo test -p world3-core --test historical_calibration`
- Change `world3-cli` references for historical calibration to `world3-core`

**Step 3: Commit**

```bash
git add docs/architecture.md CLAUDE.md
git commit -m "docs: update architecture and CLAUDE.md for validation module + test migration"
```

---

### Task 7: Regenerate traceability matrix

**Files:**
- Modify: `docs/traceability-matrix.md`

**Step 1: Update REQ tags in moved test files**

Verify that `crates/world3-core/tests/qualitative_dynamics.rs` has `// REQ: REQ-001` at the top (set in Task 3).
Verify that `crates/world3-core/tests/historical_calibration.rs` has `// REQ: REQ-026` at the top (preserved from original).

**Step 2: Regenerate the matrix**

Run: `python3 scripts/traceability.py`
Expected: exits 0 (or exits non-zero only for pre-existing coverage gaps, not for the REQs we moved).

**Step 3: Commit**

```bash
git add docs/traceability-matrix.md
git commit -m "docs: regenerate traceability matrix after test migration"
```

---

### Task 8: Final verification

**Step 1: Run full workspace tests**

Run: `cargo test --workspace`
Expected: all pass.

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings.

**Step 3: Run CLI validate**

Run: `cargo run --bin world3-cli -- validate`
Expected: all checks PASS.

**Step 4: Verify no test files remain in world3-cli**

Run: `ls crates/world3-cli/tests/`
Expected: directory is empty or doesn't exist.
