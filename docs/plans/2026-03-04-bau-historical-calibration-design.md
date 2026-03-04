# BAU Historical Calibration Regression Test — Design

## Problem

The BAU simulation is validated against qualitative Meadows 1972 dynamics (peak ranges, decline shapes) but never compared quantitatively to real-world historical data. We have historical CSVs covering ~1960-2023 for 6 variables. The simulation should track reality over the observed period — divergence indicates calibration issues.

## Requirement

**REQ-026**: The BAU simulation output shall remain within acceptable RMSE% thresholds of real-world historical data for the overlapping time period (~1960-2023) on 4 variables: Population, Food/capita, Industrial output/capita, NNR fraction.

**Current expectation**: The test will FAIL, exposing the calibration gap between simulation and reality. This is intentional — the requirement documents the aspiration, and the failing test quantifies the gap.

## Scope

### Variables tested

| Variable | Sim field | CSV file | Units |
|----------|-----------|----------|-------|
| Population | `population.population` | `population.csv` | persons |
| Food/capita | `agriculture.food_per_capita` | `food.csv` | kg/person/yr |
| IOPC | `capital.industrial_output_per_capita` | `industrial.csv` | 1975 USD/person/yr |
| NNR fraction | `resources.fraction_remaining` | `resources.csv` | fraction 0-1 |

### Variables excluded

- **Pollution**: CO2 index (historical) vs model pollution index (sim) — different quantities
- **Life expectancy**: Not in diagnose's tracked variable set
- **Services/capita**: No historical CSV

## Metric

**RMSE% of mean** = `RMSE(sim, hist) / mean(hist) * 100`

Where RMSE is computed over all years present in both simulation output and historical CSV. Since BAU runs at dt=1.0 with integer years, and historical CSVs use integer years, no interpolation is needed — just match by year.

## Thresholds (aspirational)

| Variable | RMSE% threshold | Rationale |
|----------|----------------|-----------|
| Population | < 15% | Best-calibrated variable; World 3 was designed around population dynamics |
| Food/capita | < 25% | Wider tolerance — food model is simplified (no LFH, no processing loss) |
| IOPC | < 30% | Industrial model uses aggregate capital; real-world measurement varies |
| NNR fraction | < 20% | Resource depletion is well-constrained by cumulative extraction data |

These thresholds are set to what "reasonable calibration" would look like. The current model is expected to fail some or all of them.

## Test location

`crates/world3-cli/tests/historical_calibration.rs` — a dedicated integration test file.

### Structure

- One `#[test]` per variable for granular failure reporting
- Shared helper: `load_historical_csv(path) -> Vec<(f64, f64)>`
- Shared helper: `compute_rmse_pct(sim_years, sim_values, hist_years, hist_values) -> f64`
- Each test prints actual RMSE% on failure for immediate visibility

### Dependencies

- `world3-cli` (for `diagnose::run_sim`)
- `world3-core` (for `WorldState` field access)
- No additional crate dependencies

## Bi-traceability matrix

| Requirement | Test function | File |
|------------|---------------|------|
| REQ-026 (Population) | `bau_population_tracks_historical` | `tests/historical_calibration.rs` |
| REQ-026 (Food/capita) | `bau_food_per_capita_tracks_historical` | `tests/historical_calibration.rs` |
| REQ-026 (IOPC) | `bau_iopc_tracks_historical` | `tests/historical_calibration.rs` |
| REQ-026 (NNR fraction) | `bau_nnr_fraction_tracks_historical` | `tests/historical_calibration.rs` |

## Future evolution

- When calibration improves, tighten thresholds and flip tests from `#[should_panic]` / `#[ignore]` to passing
- Add pollution and life-expectancy when unit alignment is resolved
- Consider per-decade RMSE% breakdown for targeted debugging
