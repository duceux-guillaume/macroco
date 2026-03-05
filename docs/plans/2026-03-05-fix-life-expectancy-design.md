# Fix Life Expectancy Calibration — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align life expectancy (LE) with historical data (1960–2023) by fixing two structural deviations from the pyworld3 World3-03 reference, then fine-tune with Nebel et al. (2024) insights.

**Architecture:** Replace the simplified crowding lookup with the World3-03 `LMC = 1 - CMI(IOPC) × FPU(POP)` formula (two tables). Replace LMHS1 (max 1.8) with LMHS2 (max 2.0, post-1940 default). Replace custom `FSH` fraction table with the World3-03 `HSAPC` table mapping SOPC → health allocation. Add 20-year smooth on EHSPC. Fine-tune parameters per Nebel et al. (2024) findings. Add LE historical calibration tests. Update audit report.

**Tech Stack:** Rust (world3-core lookup tables, population sector), Rust integration tests (world3-cli)

**Key references:**
- pyworld3 source: `https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/population.py`
- pyworld3 tables: `https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json`
- Nebel et al. 2024: "Recalibration of limits to growth" (DOI: 10.1111/jiec.13442)
- Nebel 2025 correction: DOI: 10.1111/jiec.70042

**Current state (BAU, sim vs historical):**

| Year | Historical LE | Simulated LE | Gap |
|------|--------------|-------------|-----|
| 1960 | 50.9 | 58.7 | +7.8 |
| 1990 | 65.1 | 58.1 | -7.0 |
| 2010 | 70.7 | 57.2 | -13.5 |
| 2020 | 72.2 | 56.3 | -15.9 |

**Root causes identified:**
1. LMCR uses simplified `lookup(pop/3.6B)` giving 0.83 at 8B pop. World3-03 uses `1 - CMI(IOPC) × FPU(POP)` giving 1.056 at same inputs (CMI goes negative at IOPC 400-800).
2. LMHS uses LMHS1 table (max 1.8) instead of LMHS2 (max 2.0, active post-1940).
3. Health services computed as `SOPC × FSH(SOPC/100)` instead of World3-03's `HSAPC(SOPC)` with 20yr smooth.
4. Theoretical LE ceiling (28 × 1.4 × 1.8 = 70.6) is below historical 73yr.

---

### Task 1: Add LE Historical Calibration Test (RED)

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`
- Reference: `data/historical/life-expectancy.csv`

**Step 1: Add LE RMSE and max-year-error tests**

Add two tests after the existing NNR tests. Use generous initial thresholds (will tighten after implementation):

```rust
/// REQ-026: BAU life expectancy must track World Bank SP.DYN.LE00.IN within 25% RMSE.
#[test]
fn bau_life_expectancy_tracks_historical() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, _years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let pct = rmse_pct(&sim_vals, &hist_vals);
    assert!(
        pct < 25.0,
        "REQ-026 Life expectancy: RMSE% = {:.1}%, threshold = 25.0%",
        pct
    );
}

/// REQ-026: BAU life expectancy max per-year error must be <= 40%.
#[test]
fn bau_life_expectancy_max_year_error() {
    let sim = bau_sim();
    let hist = load_historical_csv(&historical_dir().join("life-expectancy.csv"));
    let (sim_vals, hist_vals, years) =
        match_years(sim, |s| s.population.life_expectancy, &hist);
    let (max_err, worst_year) = max_year_error_pct(&sim_vals, &hist_vals, &years);
    assert!(
        max_err <= 40.0,
        "REQ-026 Life expectancy max-year: {:.1}% in year {} (threshold 40.0%)",
        max_err, worst_year
    );
}
```

Also add life expectancy to the `calibration_summary_report` test's `vars` vector:
```rust
("Life expect.", "life-expectancy.csv", (|s: &WorldState| s.population.life_expectancy) as fn(&WorldState) -> f64, 25.0, 40.0),
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture`
Expected: FAIL — current LE RMSE% is ~15-20% which may pass, but max-year-error at ~24% in 2022 will confirm the gap. If both pass at these generous thresholds, that's OK — we'll tighten after fixing the model.

**Step 3: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs
git commit -m "test: add life expectancy historical calibration tests (REQ-026)"
```

---

### Task 2: Replace Crowding Lookup with CMI × FPU (World3-03)

**Files:**
- Modify: `crates/world3-core/src/lookup/tables.rs` (struct + load)
- Modify: `crates/world3-core/src/model/sectors/population.rs` (formula)

**Step 2a: Update lookup table struct**

In `tables.rs`, replace the single `life_exp_multiplier_crowding` field with two new tables:

```rust
    /// Crowding multiplier from industrialization (CMI) — World3-03
    /// x: industrial output per capita [USD/person/yr]
    /// y: crowding multiplier index (can be negative at mid-IOPC)
    pub crowding_multiplier_ind: LookupTable,

    /// Fraction of population urban (FPU) — World3-03
    /// x: total population [persons]
    /// y: fraction urban [0..1]
    pub fraction_population_urban: LookupTable,
```

Remove `life_exp_multiplier_crowding`.

In the `load()` / `WorldLookupTables::new()` impl, replace the crowding table with:

```rust
            // CMI — Crowding Multiplier from Industrialization (World3-03)
            // At low IOPC (<200): crowding is severe (CMI=0.5).
            // At mid IOPC (400-800): industrialization REDUCES crowding mortality
            // (sanitation, infrastructure) → CMI goes negative → LMC > 1.0.
            // At high IOPC (>1000): crowding effect returns (urban density).
            crowding_multiplier_ind: LookupTable::new(
                "crowding_multiplier_ind",
                vec![0.0, 200.0, 400.0, 600.0, 800.0, 1000.0, 1200.0, 1400.0, 1600.0],
                vec![0.5, 0.05, -0.1, -0.08, -0.02, 0.05, 0.1, 0.15, 0.2],
            ),

            // FPU — Fraction of Population Urban (World3-03)
            // Maps total population to urbanization fraction.
            fraction_population_urban: LookupTable::new(
                "fraction_population_urban",
                vec![0.0, 2e9, 4e9, 6e9, 8e9, 10e9, 12e9, 14e9, 16e9],
                vec![0.0, 0.2, 0.4, 0.5, 0.58, 0.65, 0.72, 0.78, 0.80],
            ),
```

**Step 2b: Update population sector formula**

In `population.rs`, replace:
```rust
    let crowding_ratio = pop / CROWDING_REFERENCE_POP;
    // ...
    let lem_crowding = tables.life_exp_multiplier_crowding.eval(crowding_ratio);
```

With:
```rust
    // World3-03: LMC = 1 - CMI(IOPC) × FPU(POP)
    let cmi = tables.crowding_multiplier_ind.eval(state.capital.industrial_output_per_capita);
    let fpu = tables.fraction_population_urban.eval(pop);
    let lem_crowding = 1.0 - cmi * fpu;
```

Remove `const CROWDING_REFERENCE_POP`.

**Step 2c: Fix unit tests in population.rs**

Update `test_life_expectancy_components()` to use the new formula:
```rust
        let cmi = tables.crowding_multiplier_ind.eval(s.capital.industrial_output_per_capita);
        let fpu = tables.fraction_population_urban.eval(s.population.population.max(1.0));
        let lem_crowding = 1.0 - cmi * fpu;
```

Any other test referencing `life_exp_multiplier_crowding` or `CROWDING_REFERENCE_POP` must be updated similarly.

**Step 2d: Build and run tests**

Run: `cargo test -p world3-core`
Expected: PASS (unit tests)

Run: `cargo test -p world3-cli --test qualitative_dynamics`
Expected: May need bound adjustments — LE peak will be higher now. Check output.

**Step 2e: Commit**

```bash
git add crates/world3-core/src/lookup/tables.rs crates/world3-core/src/model/sectors/population.rs
git commit -m "fix: replace crowding lookup with World3-03 CMI×FPU formula"
```

---

### Task 3: Replace FSH + LMHS1 with HSAPC + LMHS2 (World3-03)

**Files:**
- Modify: `crates/world3-core/src/lookup/tables.rs` (replace FSH, update LMHS)
- Modify: `crates/world3-core/src/model/sectors/population.rs` (health services calc)

**Step 3a: Replace FSH table with HSAPC**

In `tables.rs` struct, replace `fraction_services_health` with:

```rust
    /// Health services allocations per capita (HSAPC) — World3-03
    /// x: service output per capita [USD/person/yr]
    /// y: health spending per capita [USD/person/yr]
    pub health_services_per_capita: LookupTable,
```

In `load()`, replace the FSH table with:

```rust
            // HSAPC — Health Services Allocations Per Capita (World3-03)
            // Maps service output per capita to health spending directly.
            // Saturates: health spending can't exceed ~$230/person even at
            // very high service output.
            health_services_per_capita: LookupTable::new(
                "health_services_per_capita",
                vec![0.0, 250.0, 500.0, 750.0, 1000.0, 1250.0, 1500.0, 1750.0, 2000.0],
                vec![0.0, 20.0, 50.0, 95.0, 140.0, 175.0, 200.0, 220.0, 230.0],
            ),
```

**Step 3b: Update LMHS to LMHS2 values**

Replace the LMHS table y-values with LMHS2 (post-1940, which is active for all BAU years):

```rust
            // LMHS — Life Expectancy Multiplier from Health Services (World3-03 LMHS2)
            // Post-1940 table (iphst=1940). Higher ceiling than LMHS1 (2.0 vs 1.8)
            // reflecting modern medical technology impact on longevity.
            life_exp_multiplier_health: LookupTable::new(
                "life_exp_multiplier_health",
                vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0],
                vec![1.0, 1.4, 1.6, 1.8, 1.95, 2.0],
            ),
```

**Step 3c: Update population sector to use HSAPC**

In `population.rs`, replace:
```rust
    let health_fraction = tables.fraction_services_health.eval(
        state.capital.service_output_per_capita / 100.0,
    );
    let health_services = state.capital.service_output_per_capita
        * health_fraction
        * params.health_investment_multiplier;
```

With:
```rust
    // World3-03: HSAPC table maps SOPC → health spending per capita directly.
    // health_investment_multiplier scales the result for scenario interventions.
    let health_services = tables.health_services_per_capita.eval(
        state.capital.service_output_per_capita,
    ) * params.health_investment_multiplier;
```

**Step 3d: Fix unit tests**

Update `test_life_expectancy_components()` to match the new health services formula:
```rust
        let health_services = tables.health_services_per_capita.eval(
            s.capital.service_output_per_capita,
        ) * params.health_investment_multiplier;
```

**Step 3e: Build and run tests**

Run: `cargo test -p world3-core`
Expected: PASS

Run: `cargo test -p world3-cli --test qualitative_dynamics`
Expected: Check — LE will now be significantly higher. Bounds may need adjustment.

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture`
Expected: Check all metrics — LE should improve; other variables should be watched.

**Step 3f: Commit**

```bash
git add crates/world3-core/src/lookup/tables.rs crates/world3-core/src/model/sectors/population.rs
git commit -m "fix: use World3-03 HSAPC table and LMHS2 for health services"
```

---

### Task 4: Add EHSPC Smooth (20yr Health Services Delay)

**Files:**
- Modify: `crates/world3-core/src/model/state.rs` (add ODE stock)
- Modify: `crates/world3-core/src/model/sectors/population.rs` (delay logic)
- Modify: `crates/world3-core/src/model/derivatives.rs` (assembly)

**Step 4a: Add `ehspc` ODE stock to WorldState**

In `state.rs`, add to `PopulationState`:
```rust
    /// Effective health services per capita — smooth of HSAPC with 20yr delay
    /// World3-03: EHSPC = smooth(HSAPC, HSID=20)
    pub ehspc: f64,
```

Update `WorldState::N` from 20 to 21.

Update `to_vec()` to include `self.population.ehspc` and `from_vec()` to read it back.

Update `Add` and `Mul` impls.

Update `initial_1900()`: set `ehspc` to `HSAPC(SOPC_1900)`. At SOPC=90: `HSAPC(90) ≈ 7.2`. Set `ehspc: 7.2`.

**Step 4b: Add EHSPC derivative in population sector**

In `population.rs`, add to `PopulationDerivatives`:
```rust
    pub d_ehspc: f64,
```

After computing `health_services` (the HSAPC value), add the smooth:
```rust
    // World3-03: EHSPC = smooth(HSAPC, HSID=20yr)
    // First-order exponential smooth: d_ehspc/dt = (HSAPC - EHSPC) / HSID
    let hsapc = tables.health_services_per_capita.eval(
        state.capital.service_output_per_capita,
    ) * params.health_investment_multiplier;
    let d_ehspc = (hsapc - state.population.ehspc) / HEALTH_SERVICES_IMPACT_DELAY;

    // Use smoothed EHSPC for life expectancy (not raw HSAPC)
    let lem_health = tables.life_exp_multiplier_health.eval(state.population.ehspc);
```

Add constant:
```rust
/// Health services impact delay [years] — World3-03: HSID = 20
const HEALTH_SERVICES_IMPACT_DELAY: f64 = 20.0;
```

**Step 4c: Wire into derivatives assembly**

In `derivatives.rs`, add the `d_ehspc` to the state vector assembly, matching the new index in `to_vec()`/`from_vec()`.

**Step 4d: Run tests**

Run: `cargo test -p world3-core && cargo test -p world3-cli --test qualitative_dynamics && cargo test -p world3-cli --test historical_calibration -- --nocapture`

The 20yr smooth will slow down early LE gains — this should reduce the 1960 overshoot (currently +7.8yr). Check the summary report.

**Step 4e: Commit**

```bash
git add crates/world3-core/src/model/state.rs crates/world3-core/src/model/sectors/population.rs crates/world3-core/src/model/derivatives.rs
git commit -m "feat: add EHSPC 20yr smooth for health services delay (World3-03)"
```

---

### Task 5: Nebel-Inspired Fine-Tuning

**Context:** Nebel et al. (2024) found that recalibrating 35 World3 parameters to match 2022 empirical data raised peaks and shifted them forward. The three parameters with the largest relative changes were: industrial capital lifetime, pollution transmission delay, and urban-industrial land development time. We cannot access the exact values behind the paywall, but we can apply the same *methodology* — adjust these high-impact parameters within plausible ranges to improve LE fit without degrading other variables.

**Files:**
- Modify: `crates/world3-core/src/model/params.rs` (BAU defaults)
- Modify: `data/presets/business_as_usual.json`

**Step 5a: Diagnostic baseline**

Run: `cargo run --bin world3-cli -- diagnose --preset bau`

Record current LE trajectory and all variable peaks.

**Step 5b: Tune parameters iteratively**

The Nebel methodology adjusts parameters within ±50% of defaults. Focus on:

1. **Pollution transmission delay** (`pollution_transmission_delay`): Nebel found this has large impact. Increasing it delays pollution's effect on LE, keeping LE higher longer.
2. **Industrial capital lifetime** (`industrial_depreciation_rate`): Affects how fast capital grows → affects SOPC → health services → LE.
3. **DCFS recalibration**: With the new higher LE from Tasks 2-4, CMPLE will change (perceived LE is higher → less compensatory fertility). DCFS may need adjustment to maintain population fit.

For each parameter change:
- Run `cargo test -p world3-cli --test historical_calibration -- --nocapture`
- Check that Population RMSE stays <14%, Food <19%, IOPC <21%, NNR <10%
- Check that LE RMSE improves

**Step 5c: Update BAU preset**

Update both `ScenarioParams::bau()` in `params.rs` and `data/presets/business_as_usual.json` to match.

**Step 5d: Commit**

```bash
git add crates/world3-core/src/model/params.rs data/presets/business_as_usual.json
git commit -m "fix: Nebel-inspired BAU parameter tuning for LE calibration"
```

---

### Task 6: Tighten Test Bounds

**Files:**
- Modify: `crates/world3-cli/tests/historical_calibration.rs`
- Modify: `crates/world3-cli/tests/qualitative_dynamics.rs`

**Step 6a: Measure final metrics**

Run: `cargo test -p world3-cli --test historical_calibration -- --nocapture`

Record exact RMSE% and max-year-error for all 5 variables.

**Step 6b: Tighten LE thresholds**

In `historical_calibration.rs`, replace the generous 25%/40% thresholds with actual + ~2% margin:
```rust
// Example: if LE RMSE = 8.5%, set threshold to 11%
// Example: if LE max-year = 18%, set threshold to 21%
```

**Step 6c: Update existing variable thresholds if improved**

If other variables improved (e.g., population RMSE dropped due to better fertility dynamics), tighten their bounds too. Never loosen existing bounds without documenting why.

**Step 6d: Update qualitative_dynamics.rs**

LE peak bounds (`45-80`) and decline threshold (`< 80% of peak`) may need updating for the new higher LE values.

**Step 6e: Run full test suite**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: ALL PASS

**Step 6f: Commit**

```bash
git add crates/world3-cli/tests/historical_calibration.rs crates/world3-cli/tests/qualitative_dynamics.rs
git commit -m "test: tighten LE calibration bounds and update qualitative dynamics"
```

---

### Task 7: Update Audit Report

**Files:**
- Modify: `docs/audit.md`

**Step 7a: Update LMCR entry**

Change status from "Intentional Deviation (structural)" to either "Exact Match" or "Aligned". Update the table to show CMI and FPU as separate exact-match tables. Remove the old single-lookup description.

**Step 7b: Update LMHS entry**

Change from "Exact Match (with LMHS1)" to "Exact Match (with LMHS2)". Note that we now use the post-1940 table as World3-03 does for BAU.

**Step 7c: Update FSH → HSAPC entry**

Change FSH from "Custom / No Direct Reference" to an HSAPC entry with "Exact Match". Show the table values side-by-side.

**Step 7d: Add EHSPC note**

Add a note about the EHSPC 20yr smooth in the population sector section, referencing World3-03's HSID=20 parameter.

**Step 7e: Update Missing Tables section**

Remove HSAPC, FPU, CMI from the "Missing pyworld3 Tables" list since they're now implemented.

**Step 7f: Add Nebel reference**

Add a new section "Calibration References" at the top or bottom citing:
- pyworld3: `https://github.com/cvanwynsberghe/pyworld3`
- Nebel et al. 2024: DOI 10.1111/jiec.13442
- Nebel 2025 correction: DOI 10.1111/jiec.70042
- Herrington 2021: "Update to limits to growth"

**Step 7g: Update Summary table**

Recount exact matches (should increase by 3: CMI, FPU, HSAPC). Update deviation count (LMCR removed). Update custom count (FSH removed).

**Step 7h: Update Recommendations**

Remove recommendation #1 about policy-switch tables for LMHS (now using LMHS2). Update as needed.

**Step 7i: Commit**

```bash
git add docs/audit.md
git commit -m "docs: update audit report for LE calibration changes"
```

---

### Task 8: Update CLAUDE.md and Product Requirements

**Files:**
- Modify: `CLAUDE.md` (simulation engine section, calibration thresholds)
- Modify: `docs/product-requirements.md` (REQ-026 update)

**Step 8a: Update CLAUDE.md**

- Update the "Historical Calibration (REQ-026)" section to include LE thresholds
- Update "LMHS" references to note LMHS2 usage
- Update the bullet about LMC/crowding to describe CMI×FPU formula
- Add EHSPC smooth to the list of ODE stocks (N=21)
- Update the FSH → HSAPC description

**Step 8b: Update REQ-026 in product-requirements.md**

Add life expectancy to the list of calibrated variables with its thresholds.

**Step 8c: Run traceability**

Run: `python3 scripts/traceability.py`

**Step 8d: Commit**

```bash
git add CLAUDE.md docs/product-requirements.md docs/traceability-matrix.md
git commit -m "docs: update CLAUDE.md and REQ-026 for LE calibration"
```

---

## Risk Assessment

1. **Population overshoot**: Higher LE → lower mortality → more population growth → may break Pop RMSE <14%. Mitigation: DCFS recalibration in Task 5.
2. **IOPC impact**: More population → lower per-capita output → IOPC RMSE may worsen. Mitigation: monitor during Task 5.
3. **Qualitative dynamics**: Higher LE peak may require wider bounds. The overshoot-and-collapse pattern should be preserved since LE still declines when pollution rises and resources deplete.
4. **N=21 ODE stocks**: Adding EHSPC changes the state vector. All `to_vec()`/`from_vec()`/`Add`/`Mul` impls must be updated consistently. This is the highest-risk mechanical change.

## Acceptance Criteria

- [ ] LE RMSE% against historical data is below tightened threshold (target: <15%)
- [ ] LE max-year-error is below tightened threshold (target: <25%)
- [ ] All 4 existing variable calibration tests still pass (Pop, Food, IOPC, NNR)
- [ ] All 5 qualitative dynamics tests pass
- [ ] Audit report (`docs/audit.md`) updated with CMI, FPU, HSAPC exact matches and Nebel reference
- [ ] `cargo test --workspace && cargo clippy --workspace -- -D warnings` passes
- [ ] CLAUDE.md updated with new ODE stock count and calibration thresholds
