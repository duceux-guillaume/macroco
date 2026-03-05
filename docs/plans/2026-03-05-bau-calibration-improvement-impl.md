# BAU Calibration Improvement — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve BAU historical fitness by ~5% across all variables via pyworld3 structural alignment (Delay3 delays, DCFS table fix), replacing the `validate` CLI with qualitative dynamics tests.

**Architecture:** Three structural fixes (Delay3 perceived LE, Delay3 pollution appearance, DCFS table alignment) plus tightened regression thresholds. State vector grows from 16 → 20 ODE stocks. The `validate` CLI command moves into a test file with wide timing windows.

**Tech Stack:** Rust (world3-core, world3-cli), RK4 ODE solver, cargo test

**Design:** `docs/plans/2026-03-05-bau-calibration-improvement-design.md`

---

### Task 1: Create qualitative dynamics test file

Move the `validate` logic from `main.rs` into a proper test file with wide timing windows.

**Files:**
- Create: `crates/world3-cli/tests/qualitative_dynamics.rs`
- Test: itself

**Step 1: Write the test file**

```rust
// REQ: REQ-001
//! BAU Qualitative Dynamics Tests
//!
//! Ensures the BAU scenario produces the Limits to Growth overshoot-and-collapse
//! pattern without constraining specific years from the 1972 study.

use std::sync::OnceLock;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

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

/// BAU population must peak between 2020-2070 then decline.
#[test]
fn bau_population_peaks_then_declines() {
    let sim = bau_sim();
    let (peak_pop, peak_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.population > mp { (s.population.population, s.time) } else { (mp, my) }
        });

    assert!(peak_pop >= 5.0e9, "Population peak {:.2e} should be >= 5B", peak_pop);
    assert!(peak_pop <= 12.0e9, "Population peak {:.2e} should be <= 12B", peak_pop);
    assert!(peak_year >= 2020.0, "Population peak year {:.0} should be >= 2020", peak_year);
    assert!(peak_year <= 2070.0, "Population peak year {:.0} should be <= 2070", peak_year);

    let pop_2100 = sim.states.iter().find(|s| (s.time - 2100.0).abs() < 0.5)
        .expect("missing year 2100").population.population;
    assert!(pop_2100 < peak_pop * 0.95,
        "2100 pop {:.2e} should be < 95% of peak {:.2e} (decline)", pop_2100, peak_pop);
}

/// BAU IOPC must peak then collapse (2100 IOPC < 50% of peak).
#[test]
fn bau_iopc_peaks_then_collapses() {
    let sim = bau_sim();
    let (peak_iopc, peak_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.capital.industrial_output_per_capita > mp {
                (s.capital.industrial_output_per_capita, s.time)
            } else { (mp, my) }
        });

    assert!(peak_year >= 2000.0, "IOPC peak year {:.0} should be >= 2000", peak_year);
    assert!(peak_year <= 2060.0, "IOPC peak year {:.0} should be <= 2060", peak_year);

    let iopc_2100 = sim.states.iter().find(|s| (s.time - 2100.0).abs() < 0.5)
        .expect("missing year 2100").capital.industrial_output_per_capita;
    assert!(iopc_2100 < peak_iopc * 0.5,
        "2100 IOPC {:.0} should be < 50% of peak {:.0} (collapse)", iopc_2100, peak_iopc);
}

/// BAU NNR must monotonically decline to < 25% by 2100.
#[test]
fn bau_nnr_monotonic_depletion() {
    let sim = bau_sim();
    let nnr_2100 = sim.states.iter().find(|s| (s.time - 2100.0).abs() < 0.5)
        .expect("missing year 2100").resources.fraction_remaining;
    assert!(nnr_2100 < 0.25,
        "2100 NNR fraction {:.3} should be < 0.25", nnr_2100);

    let monotonic = [1920.0, 1940.0, 1960.0, 1980.0, 2000.0, 2020.0, 2040.0, 2060.0, 2080.0, 2100.0]
        .windows(2)
        .all(|pair| {
            let a = sim.states.iter().find(|s| (s.time - pair[0]).abs() < 0.5)
                .expect("missing NNR state").resources.fraction_remaining;
            let b = sim.states.iter().find(|s| (s.time - pair[1]).abs() < 0.5)
                .expect("missing NNR state").resources.fraction_remaining;
            b <= a + 0.001
        });
    assert!(monotonic, "NNR fraction should decrease monotonically");
}

/// BAU life expectancy must peak (45-80yr) then decline.
#[test]
fn bau_life_expectancy_peaks_then_declines() {
    let sim = bau_sim();
    let (peak_le, _peak_year) = sim.states.iter()
        .filter(|s| s.time >= 1910.0)
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.life_expectancy > mp {
                (s.population.life_expectancy, s.time)
            } else { (mp, my) }
        });

    assert!(peak_le >= 45.0, "Peak LE {:.1} should be >= 45", peak_le);
    assert!(peak_le <= 80.0, "Peak LE {:.1} should be <= 80", peak_le);

    let le_2100 = sim.states.iter().find(|s| (s.time - 2100.0).abs() < 0.5)
        .expect("missing year 2100").population.life_expectancy;
    assert!(le_2100 < peak_le * 0.8,
        "2100 LE {:.1} should be < 80% of peak {:.1} (decline)", le_2100, peak_le);
}

/// BAU pollution must peak above 1.0 (above 1970 baseline level).
#[test]
fn bau_pollution_peaks_above_baseline() {
    let sim = bau_sim();
    let max_pollution = sim.states.iter()
        .map(|s| s.pollution.pollution_index)
        .fold(0.0_f64, f64::max);
    assert!(max_pollution > 1.0,
        "Peak pollution {:.2} should exceed 1.0 (1970 baseline)", max_pollution);
}
```

**Step 2: Run to verify it passes (baseline — before any model changes)**

Run: `cargo test -p world3-cli --test qualitative_dynamics -- --nocapture`
Expected: All 5 tests PASS (current model already produces overshoot-and-collapse)

**Step 3: Commit**

```
git add crates/world3-cli/tests/qualitative_dynamics.rs
git commit -m "test: add qualitative dynamics tests for BAU overshoot-and-collapse"
```

---

### Task 2: Remove `validate` CLI command

**Files:**
- Modify: `crates/world3-cli/src/main.rs`

**Step 1: Remove three sections from main.rs**

1. Remove `Validate` variant from `Commands` enum (line 54):
   ```rust
   // DELETE this line:
   /// Validate BAU run against Meadows 1972 reference checkpoints
   Validate,
   ```

2. Remove the match arm (lines 130-132):
   ```rust
   // DELETE these lines:
   Commands::Validate => {
       validate()?;
   }
   ```

3. Remove the entire `validate()` function (lines 354-557).

**Step 2: Run tests to verify nothing breaks**

Run: `cargo build -p world3-cli && cargo test -p world3-cli`
Expected: Compiles and all tests pass. `validate` subcommand no longer available.

**Step 3: Commit**

```
git add crates/world3-cli/src/main.rs
git commit -m "refactor: remove validate CLI command (replaced by qualitative_dynamics test)"
```

---

### Task 3: Tighten historical calibration thresholds (RED)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`

**Step 1: Update thresholds in test assertions and summary report**

Change these threshold values:

| Test | Old RMSE% | New RMSE% | Old Max-Year | New Max-Year |
|------|-----------|-----------|--------------|--------------|
| `bau_population_tracks_historical` | 16.0 | 11.0 | — | — |
| `bau_food_per_capita_tracks_historical` | 22.0 | 17.0 | — | — |
| `bau_iopc_tracks_historical` | 23.0 | 18.0 | — | — |
| `bau_nnr_fraction_tracks_historical` | 15.0 | 10.0 | — | — |
| `bau_population_max_year_error` | — | — | 42.0 | 37.0 |
| `bau_food_per_capita_max_year_error` | — | — | 30.0 | 25.0 |
| `bau_iopc_max_year_error` | — | — | 43.0 | 38.0 |
| `bau_nnr_fraction_max_year_error` | — | — | 30.0 | 25.0 |

Also update the summary report thresholds in `calibration_summary_report` (line 259-264) to match.

Also update the doc comments above each test to reflect new thresholds.

**Step 2: Run tests to verify they fail (RED)**

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture`
Expected: 7 of 8 tests FAIL (NNR RMSE 7.1% < 10.0% still passes). This is the RED state.

**Step 3: Commit**

```
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: tighten BAU calibration thresholds ~5% (RED)"
```

---

### Task 4: Add Delay3 ODE stocks to WorldState (N=16→20)

**Files:**
- Modify: `crates/world3-core/src/model/state.rs`

**Step 1: Write a failing test**

Add to existing `mod tests` in state.rs:

```rust
#[test]
fn test_world_state_n_20() {
    assert_eq!(WorldState::N, 20);
    let s = WorldState::initial_1900();
    assert_eq!(s.to_vec().len(), 20);
}
```

Run: `cargo test -p world3-core -- test_world_state_n_20`
Expected: FAIL (`assert_eq!(16, 20)`)

**Step 2: Add fields to PopulationState**

In `PopulationState` struct (after `perceived_le` at line 50), add:

```rust
    /// Delay3 stage 1 for perceived LE [years]
    pub perceived_le_stage1: f64,
    /// Delay3 stage 2 for perceived LE [years]
    pub perceived_le_stage2: f64,
```

**Step 3: Add fields to PollutionState**

In `PollutionState` struct (after `pollution_appearance_buffer` at line 107), add:

```rust
    /// Delay3 stage 1 for pollution appearance [pollution units]
    pub pollution_appearance_stage1: f64,
    /// Delay3 stage 2 for pollution appearance [pollution units]
    pub pollution_appearance_stage2: f64,
```

**Step 4: Update N, to_vec, from_vec**

Change `pub const N: usize = 16;` → `pub const N: usize = 20;`

In `to_vec()`, replace the single `perceived_le` line (181) and add:
```rust
            // Population delay (3 stocks: Delay3 for perceived LE)
            self.population.perceived_le,
            self.population.perceived_le_stage1,
            self.population.perceived_le_stage2,
```

And after `pollution_appearance_buffer` (line 179), add:
```rust
            self.pollution.pollution_appearance_stage1,
            self.pollution.pollution_appearance_stage2,
```

In `from_vec()`, update indices. The new order is:
```
0-3:   population cohorts
4-6:   capital (IC, SC, perceived_iopc)
7-11:  agriculture (arable, pot_arable, UIL, fertility, fpc_smooth)
12:    resources (NNR)
13-16: pollution (persistent, buffer, stage1, stage2)
17-19: population delay (perceived_le, stage1, stage2)
```

Update `from_vec()`:
```rust
        s.pollution.persistent_pollution = v[13].max(0.0);
        s.pollution.pollution_appearance_buffer = v[14].max(0.0);
        s.pollution.pollution_appearance_stage1 = v[15].max(0.0);
        s.pollution.pollution_appearance_stage2 = v[16].max(0.0);

        s.population.perceived_le = v[17].max(5.0);
        s.population.perceived_le_stage1 = v[18].max(5.0);
        s.population.perceived_le_stage2 = v[19].max(5.0);
```

**Step 5: Update Add and Mul impls**

In `Add`:
```rust
        self.population.perceived_le_stage1 += rhs.population.perceived_le_stage1;
        self.population.perceived_le_stage2 += rhs.population.perceived_le_stage2;
        // ...
        self.pollution.pollution_appearance_stage1 += rhs.pollution.pollution_appearance_stage1;
        self.pollution.pollution_appearance_stage2 += rhs.pollution.pollution_appearance_stage2;
```

In `Mul`:
```rust
        self.population.perceived_le_stage1 *= rhs;
        self.population.perceived_le_stage2 *= rhs;
        // ...
        self.pollution.pollution_appearance_stage1 *= rhs;
        self.pollution.pollution_appearance_stage2 *= rhs;
```

**Step 6: Update initial_1900()**

Set initial values for Delay3 stages — at steady state, all 3 stages equal:
```rust
perceived_le_stage1: 33.0,
perceived_le_stage2: 33.0,
```
(same as `perceived_le: 33.0`)

For pollution:
```rust
pollution_appearance_stage1: 0.05,
pollution_appearance_stage2: 0.05,
```
(same as `pollution_appearance_buffer: 0.05`)

**Step 7: Fix the old N=16 test**

Replace `test_world_state_n` and `test_returns_16_stock_derivatives`:
- Change `assert_eq!(WorldState::N, 16)` → `assert_eq!(WorldState::N, 20)`
- Change `assert_eq!(v.len(), 16)` → `assert_eq!(v.len(), 20)`
- Update comment in derivatives.rs line 31

**Step 8: Run tests**

Run: `cargo test -p world3-core`
Expected: All tests pass (new stocks exist but Delay3 logic not wired yet — old stocks still work).

**Step 9: Commit**

```
git add crates/world3-core/src/model/state.rs
git commit -m "feat: add 4 Delay3 ODE stocks to WorldState (N=16→20)"
```

---

### Task 5: Implement Delay3 for perceived LE

**Files:**
- Modify: `crates/world3-core/src/model/sectors/population.rs`
- Modify: `crates/world3-core/src/model/derivatives.rs`

**Step 1: Write a failing test for Delay3 behavior**

Add to `population.rs` tests:

```rust
    #[test]
    fn test_perceived_le_delay3_stages() {
        let (mut s, params, tables) = setup();
        // Set stages out of equilibrium
        s.population.perceived_le_stage1 = 20.0;
        s.population.perceived_le_stage2 = 20.0;
        s.population.perceived_le = 20.0;
        let d = population_derivatives(&mut s, &params, &tables);
        // Stage 1 should converge toward actual LE fastest
        assert!(d.d_perceived_le_stage1 > d.d_perceived_le_stage2,
            "stage1 rate ({}) should exceed stage2 rate ({})",
            d.d_perceived_le_stage1, d.d_perceived_le_stage2);
        // Stage 2 should converge toward stage 1
        assert!(d.d_perceived_le_stage2 > 0.0,
            "stage2 rate should be positive when stage1 > stage2");
    }
```

Run: `cargo test -p world3-core -- test_perceived_le_delay3_stages`
Expected: FAIL (fields don't exist on PopulationDerivatives yet)

**Step 2: Update PopulationDerivatives struct**

In `population.rs`, add to `PopulationDerivatives` (line 30-36):
```rust
pub struct PopulationDerivatives {
    pub d_cohort_0_14: f64,
    pub d_cohort_15_44: f64,
    pub d_cohort_45_64: f64,
    pub d_cohort_65_plus: f64,
    pub d_perceived_le: f64,
    pub d_perceived_le_stage1: f64,
    pub d_perceived_le_stage2: f64,
}
```

**Step 3: Implement Delay3 in population_derivatives**

Replace the Delay1 perceived LE computation (line 134-139):
```rust
    // ---- Perceived life expectancy (Delay3: 3 cascaded first-order stages) ----
    // World3-03: PLE = Delay3(LE, LPD=20yr). Three stages with τ = LPD/3.
    let tau = LIFETIME_PERCEPTION_DELAY / 3.0;
    let d_perceived_le_stage1 = (life_expectancy - state.population.perceived_le_stage1) / tau;
    let d_perceived_le_stage2 = (state.population.perceived_le_stage1 - state.population.perceived_le_stage2) / tau;
    let d_perceived_le = (state.population.perceived_le_stage2 - perceived_le) / tau;
```

Update the return struct to include new fields:
```rust
    PopulationDerivatives {
        d_cohort_0_14: births_per_year - aging_0_to_15 - deaths_0_14,
        d_cohort_15_44: aging_0_to_15 - aging_15_to_45 - deaths_15_44,
        d_cohort_45_64: aging_15_to_45 - aging_45_to_65 - deaths_45_64,
        d_cohort_65_plus: aging_45_to_65 - deaths_65_plus,
        d_perceived_le,
        d_perceived_le_stage1,
        d_perceived_le_stage2,
    }
```

**Step 4: Wire Delay3 stocks in derivatives.rs**

In `derivatives.rs`, after line 81, add:
```rust
    d.population.perceived_le_stage1 = pop_deriv.d_perceived_le_stage1;
    d.population.perceived_le_stage2 = pop_deriv.d_perceived_le_stage2;
```

Update the doc comment at line 1-2 to say "20 ODE stocks" instead of "16 ODE stocks".

**Step 5: Run tests**

Run: `cargo test -p world3-core`
Expected: All tests pass including new Delay3 test.

**Step 6: Commit**

```
git add crates/world3-core/src/model/sectors/population.rs crates/world3-core/src/model/derivatives.rs
git commit -m "feat: implement Delay3 for perceived LE (pyworld3 alignment)"
```

---

### Task 6: Implement Delay3 for pollution appearance

**Files:**
- Modify: `crates/world3-core/src/model/sectors/pollution.rs`
- Modify: `crates/world3-core/src/model/derivatives.rs`

**Step 1: Write a failing test**

Add to `pollution.rs` tests:

```rust
    #[test]
    fn test_pollution_delay3_pipeline() {
        let (mut s, params, tables) = setup();
        // Load stage1 with pollution, stages 2/3 empty
        s.pollution.pollution_appearance_stage1 = 1.0;
        s.pollution.pollution_appearance_stage2 = 0.0;
        s.pollution.pollution_appearance_buffer = 0.0; // stage3
        let (d_persistent, d_stage1, d_stage2, d_stage3) =
            pollution_derivatives(&mut s, &params, &tables);
        // Stage1 should be draining
        assert!(d_stage1 < 0.0 || s.pollution.generation_rate > 0.0,
            "stage1 should be draining or refilling from generation");
        // Stage2 should be filling from stage1
        assert!(d_stage2 > 0.0, "stage2 should receive flow from stage1");
    }
```

Run: `cargo test -p world3-core -- test_pollution_delay3_pipeline`
Expected: FAIL (function returns wrong tuple)

**Step 2: Change pollution_derivatives return type**

Change return type from `(f64, f64)` to `(f64, f64, f64, f64)`:
```rust
/// Returns (d_persistent_pollution, d_stage1, d_stage2, d_stage3)
pub fn pollution_derivatives(
    state: &mut WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> (f64, f64, f64, f64) {
```

**Step 3: Replace Delay1 with Delay3 pipeline**

Replace lines 51-58 (the appearance delay computation) with:
```rust
    // ---- 20-year appearance delay (Delay3: 3 cascaded pipeline stages) ----
    // World3-03: PPTD = 20yr. Delay3 gives pipeline-like transit behavior
    // (more uniform transit time than Delay1). Each stage τ = PPTD/3.
    let tau = POLLUTION_APPEARANCE_DELAY / 3.0;
    let flow_1_to_2 = state.pollution.pollution_appearance_stage1 / tau;
    let flow_2_to_3 = state.pollution.pollution_appearance_stage2 / tau;
    let appearance_rate = state.pollution.pollution_appearance_buffer / tau; // stage3 out

    let d_stage1 = generation - flow_1_to_2;
    let d_stage2 = flow_1_to_2 - flow_2_to_3;
    let d_stage3 = flow_2_to_3 - appearance_rate;
```

Update the return:
```rust
    (d_persistent, d_stage1, d_stage2, d_stage3)
```

Remove the old `d_buffer` variable.

**Step 4: Update derivatives.rs wiring**

Replace lines 69 and 95-96:
```rust
    // --- Step 5: Pollution ---
    let (d_pollution, d_poll_stage1, d_poll_stage2, d_poll_stage3) =
        pollution::pollution_derivatives(&mut s, params, tables);
    // ...
    d.pollution.persistent_pollution = d_pollution;
    d.pollution.pollution_appearance_stage1 = d_poll_stage1;
    d.pollution.pollution_appearance_stage2 = d_poll_stage2;
    d.pollution.pollution_appearance_buffer = d_poll_stage3;
```

**Step 5: Fix existing pollution tests**

Update tests that destructure the old `(f64, f64)` return to use `(f64, f64, f64, f64)`. Specifically:
- `test_pollution_appearance_delay`: `let (d_persistent, d_stage1, d_stage2, d_stage3) = ...`
- `test_steady_state_pollution`: update buffer steady-state setup (buffer = generation × τ, not generation × delay)
- `test_buffer_flow_balance`: update to check all 3 stages
- `test_high_pollution_net_accumulation`: update destructuring

Also update `test_steady_state_pollution`: at steady state with Delay3, each stage holds `generation × τ = generation × PPTD/3`. So:
```rust
s.pollution.pollution_appearance_stage1 = generation * tau;
s.pollution.pollution_appearance_stage2 = generation * tau;
s.pollution.pollution_appearance_buffer = generation * tau; // stage3
```

And check `d_stage1.abs() < generation * 0.01` (etc).

**Step 6: Fix derivatives.rs test**

In `test_symmetry_with_individual_sectors`: update the pollution destructuring from
`let (d_poll, d_poll_buf) = ...` to `let (d_poll, d_poll_s1, d_poll_s2, d_poll_s3) = ...`
and add assertions for the new stages.

**Step 7: Run tests**

Run: `cargo test -p world3-core`
Expected: All tests pass.

**Step 8: Commit**

```
git add crates/world3-core/src/model/sectors/pollution.rs crates/world3-core/src/model/derivatives.rs
git commit -m "feat: implement Delay3 for pollution appearance (pyworld3 alignment)"
```

---

### Task 7: Align DCFS table with pyworld3

**Files:**
- Modify: `crates/world3-core/src/lookup/tables.rs`

**Step 1: Write a failing test**

Add to an existing test module or create inline:

```rust
#[test]
fn test_dcfs_matches_pyworld3() {
    let tables = WorldLookupTables::load();
    // pyworld3 effective DCFS = dcfsn(3.8) × SFSN(DIOPC)
    let dcfs_0 = tables.desired_family_size.eval(0.0);
    assert!((dcfs_0 - 4.75).abs() < 0.1,
        "DCFS(0) = {} should be ~4.75 (pyworld3)", dcfs_0);
    let dcfs_200 = tables.desired_family_size.eval(200.0);
    assert!((dcfs_200 - 3.80).abs() < 0.1,
        "DCFS(200) = {} should be ~3.80 (pyworld3)", dcfs_200);
}
```

Run: `cargo test -p world3-core -- test_dcfs_matches_pyworld3`
Expected: FAIL (current values are 3.40 and 3.39)

**Step 2: Update the table**

In `tables.rs` (lines 266-267), change:
```rust
// Old:
vec![0.0, 200.0, 400.0, 600.0, 800.0],
vec![3.40, 3.39, 2.87, 2.29, 1.88],
// New (pyworld3 effective dcfsn × SFSN):
vec![0.0, 200.0, 400.0, 600.0, 800.0],
vec![4.75, 3.80, 3.42, 3.04, 2.85],
```

**Step 3: Run test**

Run: `cargo test -p world3-core -- test_dcfs_matches_pyworld3`
Expected: PASS

**Step 4: Run full workspace tests**

Run: `cargo test --workspace`
Expected: Most pass. Historical calibration tests still RED (expected). Qualitative dynamics tests should still pass — check carefully. If population now overshoots the 12B ceiling in qualitative test, we'll adjust in Task 8.

**Step 5: Commit**

```
git add crates/world3-core/src/lookup/tables.rs
git commit -m "feat: align DCFS table with pyworld3 effective values"
```

---

### Task 8: Calibration tuning (GREEN)

After structural fixes, run diagnostics and tune parameters to get tests GREEN.

**Files:**
- Possibly modify: `crates/world3-core/src/model/params.rs`
- Possibly modify: `crates/world3-core/src/lookup/tables.rs`
- Possibly modify: `data/presets/business_as_usual.json`

**Step 1: Run diagnostics to see current state**

```bash
cargo test -p world3-cli --test historical_calibration -- --nocapture
cargo test -p world3-cli --test qualitative_dynamics -- --nocapture
cargo run --bin world3-cli -- diagnose --preset bau
cargo run --bin world3-cli -- diagnose --preset bau --stability-check
```

Review the calibration report. Note which variables improved, which worsened, and by how much.

**Step 2: Iterative tuning**

Based on the diagnostic output, adjust parameters. Likely candidates:
- `technology_growth_rate` (currently 0.014) — reduce if IOPC now overshoots due to delayed collapse
- `resource_efficiency` (currently 1.0) — increase if NNR still depletes too fast
- FIOAC consumption fraction table — fine-tune cap if needed

After each parameter change:
```bash
cargo test -p world3-cli --test historical_calibration -- --nocapture
cargo test -p world3-cli --test qualitative_dynamics -- --nocapture
```

**Step 3: Verify all tests pass (GREEN)**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: All tests pass including tightened thresholds.

**Step 4: Commit**

```
git add -A
git commit -m "feat: tune BAU parameters after structural alignment (GREEN)"
```

---

### Task 9: Update documentation and CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`
- Modify: `docs/product-requirements.md` (update REQ-026 thresholds)
- Run: `python3 scripts/traceability.py`

**Step 1: Update CLAUDE.md**

- Change all references to N=16 → N=20
- Update "16 ODE stocks" → "20 ODE stocks"
- Remove references to `validate` CLI command
- Add note about Delay3 for perceived LE and pollution appearance
- Update historical calibration thresholds to new values
- Update "KNOWN DEVIATION" notes — Delay1→Delay3 is now fixed

**Step 2: Update REQ-026 thresholds in product-requirements.md**

Update the thresholds to match the new test values.

**Step 3: Regenerate traceability matrix**

Run: `python3 scripts/traceability.py`
Expected: Matrix updated with new `qualitative_dynamics.rs` test file.

**Step 4: Commit**

```
git add CLAUDE.md docs/product-requirements.md docs/traceability-matrix.md
git commit -m "docs: update CLAUDE.md and REQ-026 for Delay3 and tightened thresholds"
```

---

### Task 10: Final verification

**Step 1: Full test suite**

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo test -p world3-cli --test historical_calibration -- --nocapture
cargo test -p world3-cli --test qualitative_dynamics -- --nocapture
cargo run --bin world3-cli -- diagnose --preset bau --stability-check
```

**Step 2: Verify all presets still work**

```bash
cargo run --bin world3-cli -- simulate --preset technology --output /dev/null
cargo run --bin world3-cli -- simulate --preset stabilized --output /dev/null
```

**Step 3: Check frontend build**

```bash
cd frontend && npm run check && npm test && npm run build
```

**Step 4: Commit any final fixes, then use `/superpowers:finishing-a-development-branch`**
