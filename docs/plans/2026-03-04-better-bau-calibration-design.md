# Better BAU Calibration — Phase 2

**Date:** 2026-03-04
**REQ:** REQ-026 (Historical Calibration)
**Status:** Design

## Problem

BAU simulation diverges significantly from historical data, especially in later years:

| Variable | Max year error | Worst period | Pattern |
|---|---|---|---|
| Population | 39% (1961) | 1960s: +35% | Overshoots early, undershoots late |
| Food/capita | 35% (2022) | 2020s: -34% | Consistently underestimated, worsening |
| IOPC | 52% (2023) | 2020s: -48% | Overshoots early, collapses too soon |
| NNR fraction | 11.5% (2010) | 2000s: -11% | Systematic over-depletion |

Current RMSE% thresholds (all barely passing):
- Population: 14.1% vs 15% threshold
- Food/capita: 24.8% vs 25% threshold
- IOPC: 28.2% vs 30% threshold
- NNR: 4.3% vs 20% threshold

## Root Cause Analysis

### IOPC Collapse (primary driver)

The model reaches "high-income" consumption territory (IOPC > 400) by 1960 — roughly 50 years too early relative to real-world development. This triggers the World3 overshoot mechanism prematurely:

1. **Weak technology growth**: `technology_growth_rate = 0.002` (0.2%/yr from 1970) gives only 11% cumulative boost by 2023. Real TFP growth was ~1.5-2%/yr during 1960-2000.

2. **Static SOPC reference**: Service output per capita is normalized by a hardcoded `200.0` instead of World3-03's dynamic ISOPC lookup table. As economy grows, `SOPC/200` exceeds 1.0, driving service allocation to zero. Then service capital depreciates without reinvestment, distorting the investment fraction.

3. **FIOAC consumption squeeze**: The consumption fraction jumps from 0.43 to 0.50 between IOPC 400-480, reducing the investment fraction sharply at the exact IOPC level the sim reaches in the 1960s-1970s.

### Cascade effects

- Low IOPC → low agricultural investment (via FIOAA) → low food per capita
- Low IOPC → lower population carrying capacity → faster population decline

## Design

### Phase A: Test Changes (TDD — red first)

Add new strict test: **max per-year absolute % error** for all 4 variables.

**New thresholds:**
| Variable | Max-year threshold | RMSE% threshold (tightened) |
|---|---|---|
| Population | 30% | 12% |
| Food/capita | 30% | 20% |
| IOPC | 30% | 20% |
| NNR fraction | 30% | 15% |

Add tests to `historical_calibration.rs`:
- `bau_population_max_year_error` — max abs % error per year ≤ 30%
- `bau_food_per_capita_max_year_error` — max abs % error per year ≤ 30%
- `bau_iopc_max_year_error` — max abs % error per year ≤ 30%
- `bau_nnr_max_year_error` — max abs % error per year ≤ 30%

Also tighten existing RMSE% test thresholds.

### Phase B: Model Changes (make tests green)

#### Step 1: Increase technology_growth_rate

- Current: 0.002 → target: ~0.008-0.012 (calibrate by binary search)
- Affects: `ScenarioParams::default()`, `ScenarioParams::bau()`, `data/presets/business_as_usual.json`
- Rationale: Real TFP growth was ~1.5%/yr; the model's tech multiplier compensates for all unmodeled productivity improvements

#### Step 2: Add dynamic ISOPC lookup table

- World3-03 has `ISOPC = f(IOPC)` — indicated service output per capita grows with income
- Currently hardcoded as `200.0` in `capital.rs` (service demand normalizer)
- Add ISOPC lookup from pyworld3, use it as the denominator for service allocation
- This keeps `SOPC/ISOPC` moderate as economy grows, preventing premature service disinvestment

#### Step 3: Fine-tune FIOAC if needed

- Only if Steps 1-2 don't achieve ≤30% max-year for IOPC
- Smooth the curve above IOPC=400 more gradually
- Must be validated against pyworld3 reference trajectory

### Phase C: Cleanup

1. Re-run `/audit-tables` to update stale `docs/audit.md`
2. Update REQ-026 in `docs/product-requirements.md` with new thresholds
3. Regenerate traceability matrix with `python3 scripts/traceability.py`
4. Run full validation: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo run --bin world3-cli -- validate`

## Constraints

- Must still pass `validate` (Meadows 1972 qualitative dynamics: pop peaks ~2030, resources deplete, IOPC collapses before 2100)
- Must not break Technology or Stabilized preset behavior
- All existing tests must pass
- Changes to `ScenarioParams::default()` must be mirrored in `business_as_usual.json`

## Success Criteria

All 4 variables pass both:
1. Max per-year absolute % error ≤ 30%
2. Tightened RMSE% thresholds (Pop 12%, Food 20%, IOPC 20%, NNR 15%)

Plus: `validate` still passes, `diagnose --stability-check` shows no UNSTABLE variables.
