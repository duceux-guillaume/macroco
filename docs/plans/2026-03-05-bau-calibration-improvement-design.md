# BAU Calibration Improvement — Design

**Date:** 2026-03-05
**Requirement:** REQ-026 (Historical Calibration)
**Approach:** TDD — tighten thresholds (RED), fix structural shortcuts (GREEN)

## Problem

BAU historical fitness has razor-thin margins. IOPC RMSE is at 22.9% vs 23% threshold (0.1% margin). The `validate` CLI command enforces 1972 Meadows timing that conflicts with improving historical fitness.

Current values:

| Variable | RMSE% | Threshold | Max-Year | Threshold |
|----------|-------|-----------|----------|-----------|
| Population | 15.5% | 16.0% | 41.5% @ 1961 | 42.0% |
| Food/capita | 21.2% | 22.0% | 29.1% @ 2022 | 30.0% |
| IOPC | 22.9% | 23.0% | 42.3% @ 2023 | 43.0% |
| NNR fraction | 7.1% | 15.0% | 29.9% @ 2023 | 30.0% |

Root cause chain: NNR depletes ~30% too fast → FCAOR rises → capital starvation → IOPC collapse ~2010 → food drops → all errors grow from 2010-2023.

## Design

### Phase A: Replace `validate` CLI with qualitative dynamics test (RED)

**Remove:** `validate` subcommand from `world3-cli/src/main.rs`.

**Add:** `crates/world3-cli/tests/qualitative_dynamics.rs` — a test file that enforces BAU produces overshoot-and-collapse with wide timing windows.

Checks (migrated from validate fn, with widened windows):
- Population peaks between 2020-2070 at 5B-12B, then declines (2100 pop < 95% of peak)
- NNR monotonically decreasing; < 25% remaining by 2100
- IOPC peaks then collapses (2100 IOPC < 50% of peak)
- Life expectancy peaks 45-80yr, then declines (2100 LE < 80% of peak)
- Pollution index peaks > 1.0

These checks ensure the model still produces the Limits to Growth overshoot-and-collapse dynamic without constraining specific years or magnitudes from the 1972 study.

### Phase B: Tighten historical calibration thresholds (RED)

Update `crates/world3-cli/tests/historical_calibration.rs` with new targets (~5% absolute tighter):

| Variable | New RMSE% Threshold | New Max-Year Threshold |
|----------|---------------------|------------------------|
| Population | 11.0% | 37.0% |
| Food/capita | 17.0% | 25.0% |
| IOPC | 18.0% | 38.0% |
| NNR fraction | 10.0% | 25.0% |

Tests will fail (RED). That's the point.

### Phase C: Structural fixes — pyworld3 alignment (GREEN)

Three changes ordered by expected impact. After each change, run tests to check progress.

#### Fix 1: Delay3 for Perceived LE

**File:** `population.rs`, `state.rs`, `derivatives.rs`

Replace first-order delay with 3-stage cascaded delay:
```
d(stage1)/dt = (LE - stage1) / (LPD/3)
d(stage2)/dt = (stage1 - stage2) / (LPD/3)
d(perceived_le)/dt = (stage2 - perceived_le) / (LPD/3)
```
Where LPD = 20yr, so each stage τ = 6.67yr.

Adds 2 ODE stocks to `PopulationState`: `perceived_le_stage1`, `perceived_le_stage2`.
N goes from 16 → 18.

**Expected impact:** Perceived LE lags actual LE more (pipeline behavior vs exponential). CMPLE stays high longer → higher birth rates in early decades → better population fit for 1960s. Slower demographic transition delays population peak.

#### Fix 2: Delay3 for Pollution Appearance

**File:** `pollution.rs`, `state.rs`, `derivatives.rs`

Replace single-buffer delay with 3-stage pipeline:
```
d(stage1)/dt = generation - stage1 / (PPTD/3)
d(stage2)/dt = stage1/(PPTD/3) - stage2/(PPTD/3)
d(stage3)/dt = stage2/(PPTD/3) - stage3/(PPTD/3)
appearance_rate = stage3 / (PPTD/3)
```
Where PPTD = 20yr, so each stage τ = 6.67yr.

Replaces 1 ODE stock (`pollution_appearance_buffer`) with 3 stocks (`pollution_stage1/2/3`).
N goes from 18 → 20.

**Expected impact:** Pollution appears more uniformly after ~20yr (less early leakage). Delays pollution-driven LE decline → delays population peak. Delays FCAOR-driven collapse → better IOPC fit 2010-2023.

#### Fix 3: DCFS table alignment with pyworld3

**File:** `tables.rs`

Current DCFS values are 10-34% lower than pyworld3's effective `dcfsn × SFSN(DIOPC)`.

Current: `[3.40, 3.39, 2.87, 2.29, 1.88]` at IOPC `[0, 200, 400, 600, 800]`
pyworld3: `[4.75, 3.80, 3.42, 3.04, 2.85]` at IOPC `[0, 200, 400, 600, 800]`

Align to pyworld3 values.

**Expected impact:** Higher desired family size → more births → higher population growth. Combined with Fix 1, should improve early population trajectory (1960s model currently underestimates after crossover adjustment).

#### Fix 4: Parameter tuning (if needed)

After structural fixes, if errors persist:
- Adjust `resource_efficiency` upward from 1.0 (real-world BAU includes some efficiency gains)
- Fine-tune `technology_growth_rate` (currently 0.014) if IOPC overshoots
- These are secondary knobs after structural alignment

### What We Keep (intentional deviations from pyworld3)

- `technology_growth_rate = 0.014` — compensates for unmodeled TFP growth
- `FIOAC cap at 0.70` — prevents consumption trap (pyworld3's 0.83 stalls IOPC)
- `FIOAA floor at 0.005` — stability guard for non-BAU scenarios
- Custom pollution normalization — calibrated to our scale

### Files Modified

| File | Change |
|------|--------|
| `world3-cli/src/main.rs` | Remove `Validate` subcommand |
| `world3-cli/tests/qualitative_dynamics.rs` | New test file (from validate logic) |
| `world3-cli/tests/historical_calibration.rs` | Tighten thresholds |
| `world3-core/src/model/state.rs` | N=16→20, add 4 new ODE stocks, update to_vec/from_vec/Add/Mul |
| `world3-core/src/model/sectors/population.rs` | Delay3 for perceived LE |
| `world3-core/src/model/sectors/pollution.rs` | Delay3 for pollution appearance |
| `world3-core/src/model/derivatives.rs` | Wire new stocks into derivative assembly |
| `world3-core/src/lookup/tables.rs` | Align DCFS table to pyworld3 |
| `data/presets/business_as_usual.json` | Update if any ScenarioParams change |
| CLAUDE.md | Update N=16→20, document Delay3, update validate references |

### Verification

After each fix:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo test -p world3-cli --test historical_calibration -- --nocapture
cargo test -p world3-cli --test qualitative_dynamics -- --nocapture
```

### Success Criteria

All 8 historical calibration tests pass with tightened thresholds AND all qualitative dynamics tests pass.
