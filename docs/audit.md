# Lookup Table Audit Report

**Date:** 2026-03-05
**Auditor:** Claude (automated comparison)
**Our source:** `crates/world3-core/src/lookup/tables.rs`
**Reference:** [pyworld3 `functions_table_world3.json`](https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json) (faithfully digitized from World3-03 Vensim model, Meadows 2004)
**Secondary reference:** [WorldDynamics.jl World3 module](https://github.com/worlddynamics/WorldDynamics.jl)
**Calibration reference:** [Nebel et al. 2024 — "Recalibration of limits to growth"](https://doi.org/10.1111/jiec.13442) (DOI: 10.1111/jiec.13442), [Correction 2025](https://doi.org/10.1111/jiec.70042)

---

## Summary

| Category | Count | Tables |
|----------|------:|--------|
| Exact Match | 22 | M1, M2, M3, M4, LMF, LMHS2, LMP, FM, CMPLE, FIOAS1, IFPC1, ISOPC1, JPICU, LYMC, LYMAP1, UILPC, LFDR, LFRT, FALM, FCAOR1, HSAPC, CMI+FPU |
| Intentional Deviation | 5 | DCFS, FRSN, FIOACV, FIOAA, PPASR |
| Custom / No Reference | 6 | LFP, LERD, LDCO, FRNF, PPGIO, PPGAO |
| **Total** | **33** | |

**Risk assessment:**
- 22 exact-match tables cover all mortality, fertility, land, resource, health, crowding, and most capital dynamics faithfully.
- Major alignment since 2026-03-04: FM, CMPLE, LMF, LMHS2, LMP, IFPC, ISOPC, JPICU, LYMC, LYMAP, UILPC, LFDR, FCAOR, HSAPC, CMI, FPU all brought to exact pyworld3 match.
- 5 intentional deviations are documented calibration adjustments: DCFS (calibrated for Delay3 model), FIOACV (capped at 0.70), FIOAA (0.005 floor), FRSN (different x-axis), PPASR (different pollution normalization).
- Highest-risk deviation: DCFS — calibrated values differ significantly from pyworld3 effective DCFS to compensate for Delay3 perceived-LE dynamics.

---

## Table-by-Table Comparison

### Population Sector

#### M1 — Mortality 0-14 (`mortality_0_14`)

| | x | y |
|---|---|---|
| **Ours** | 20, 30, 40, 50, 60, 70, 80 | 0.0567, 0.0366, 0.0243, 0.0155, 0.0082, 0.0023, 0.001 |
| **pyworld3** | 20, 30, 40, 50, 60, 70, 80 | 0.0567, 0.0366, 0.0243, 0.0155, 0.0082, 0.0023, 0.001 |

**Status: Exact Match**

---

#### M2 — Mortality 15-44 (`mortality_15_44`)

| | x | y |
|---|---|---|
| **Ours** | 20, 30, 40, 50, 60, 70, 80 | 0.0266, 0.0171, 0.0110, 0.0065, 0.0040, 0.0016, 0.0008 |
| **pyworld3** | 20, 30, 40, 50, 60, 70, 80 | 0.0266, 0.0171, 0.0110, 0.0065, 0.0040, 0.0016, 0.0008 |

**Status: Exact Match**

---

#### M3 — Mortality 45-64 (`mortality_45_64`)

| | x | y |
|---|---|---|
| **Ours** | 20, 30, 40, 50, 60, 70, 80 | 0.0562, 0.0373, 0.0252, 0.0171, 0.0118, 0.0083, 0.006 |
| **pyworld3** | 20, 30, 40, 50, 60, 70, 80 | 0.0562, 0.0373, 0.0252, 0.0171, 0.0118, 0.0083, 0.006 |

**Status: Exact Match**

---

#### M4 — Mortality 65+ (`mortality_65_plus`)

| | x | y |
|---|---|---|
| **Ours** | 20, 30, 40, 50, 60, 70, 80 | 0.13, 0.11, 0.09, 0.07, 0.06, 0.05, 0.04 |
| **pyworld3** | 20, 30, 40, 50, 60, 70, 80 | 0.13, 0.11, 0.09, 0.07, 0.06, 0.05, 0.04 |

**Status: Exact Match**

---

#### LMF — Life Expectancy Multiplier from Food (`life_exp_multiplier_food`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4, 5 | 0, 1, 1.2, 1.3, 1.35, 1.4 |
| **pyworld3 (LMF)** | 0, 1, 2, 3, 4, 5 | 0, 1, 1.2, 1.3, 1.35, 1.4 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work.

---

#### LMHS — Life Expectancy Multiplier from Health Services (`life_exp_multiplier_health`)

| | x | y |
|---|---|---|
| **Ours** | 0, 20, 40, 60, 80, 100 | 1.0, 1.4, 1.6, 1.8, 1.95, 2.0 |
| **pyworld3 (LMHS1)** | 0, 20, 40, 60, 80, 100 | 1, 1.1, 1.4, 1.6, 1.7, 1.8 |
| **pyworld3 (LMHS2)** | 0, 20, 40, 60, 80, 100 | 1, 1.4, 1.6, 1.8, 1.95, 2.0 |

**Status: Exact Match (with LMHS2)**

World3-03 has two tables (LMHS1 before policy switch year iphst=1940, LMHS2 after). We use LMHS2 which reflects post-1940 modern medical technology impact on longevity. Input is EHSPC (effective health services per capita), which is a 20-year first-order smooth of HSAPC (World3-03: HSID=20yr), implemented as an ODE stock.

---

#### CMI — Crowding Multiplier from Industrialization (`crowding_multiplier_ind`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 0.5, 0.05, −0.1, −0.08, −0.02, 0.05, 0.1, 0.15, 0.2 |
| **pyworld3 (CMI)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 0.5, 0.05, −0.1, −0.08, −0.02, 0.05, 0.1, 0.15, 0.2 |

**Status: Exact Match**

#### FPU — Fraction of Population Urban (`fraction_population_urban`)

| | x | y |
|---|---|---|
| **Ours** | 0, 2e9, 4e9, 6e9, 8e9, 10e9, 12e9, 14e9, 16e9 | 0, 0.2, 0.4, 0.5, 0.58, 0.65, 0.72, 0.78, 0.8 |
| **pyworld3 (FPU)** | 0, 2e9, 4e9, 6e9, 8e9, 10e9, 12e9, 14e9, 16e9 | 0, 0.2, 0.4, 0.5, 0.58, 0.65, 0.72, 0.78, 0.8 |

**Status: Exact Match**

World3-03 computes LMC = 1 − CMI(IOPC) × FPU(POP) using two separate tables. Both tables now match pyworld3 exactly.

---

#### LMP — Life Expectancy Multiplier from Pollution (`life_exp_multiplier_pollution`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100 | 1.0, 0.99, 0.97, 0.95, 0.9, 0.85, 0.75, 0.65, 0.55, 0.4, 0.2 |
| **pyworld3 (LMP)** | 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100 | 1.0, 0.99, 0.97, 0.95, 0.90, 0.85, 0.75, 0.65, 0.55, 0.40, 0.20 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had fewer breakpoints and less severe high-pollution effects.

---

#### DCFS — Desired Completed Family Size (`desired_family_size`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800 | 2.85, 3.50, 3.00, 2.42, 1.90 |
| **pyworld3 (SFSN)** | 0, 200, 400, 600, 800 | 1.25, 1.0, 0.9, 0.8, 0.75 |
| **World3-03 effective** | DCFS = dcfsn(3.8) × SFSN | 4.75, 3.80, 3.42, 3.04, 2.85 |

**Status: Intentional Deviation (calibrated)**

We merge dcfsn × SFSN into a single table. CMPLE is applied separately (matching pyworld3 CMPLE exactly).

Differences from pyworld3 effective DCFS:
- At DIOPC=0: ours=2.85 vs pyworld3=4.75 (−40%). Lower early-1900s growth.
- At DIOPC=200: ours=3.50 vs pyworld3=3.80 (−8%). Non-monotonic hump captures mid-income population boom.
- At DIOPC=800: ours=1.90 vs pyworld3=2.85 (−33%). Faster demographic transition.

**Rationale:** Calibrated specifically for Delay3 perceived-LE dynamics. The Delay3 pipeline causes CMPLE to stay high longer than with Delay1, so base DCFS must be lower to avoid population overshoot. Full pyworld3 DCFS values caused population to spike to 14.15B (79.5% RMSE). Current values give Pop RMSE=13.2%, peak ~8.2B at ~2082.

**Impact:** High — primary fertility control. Calibrated so total fertility (DCFS × CMPLE × FM × FRSN) matches historical trajectory.

---

#### FRSN — Family Planning Multiplier (`family_planning_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.25, 0.5, 0.75, 1.0 | 1.0, 0.90, 0.75, 0.55, 0.40 |
| **pyworld3 (FRSN)** | −0.2, −0.1, 0, 0.1, 0.2 | 0.5, 0.6, 0.7, 0.85, 1.0 |

**Status: Intentional Deviation (structural)**

Different x-axis: ours uses 0–1 (planning effectiveness), pyworld3 uses family income expectation difference. Different mechanism but comparable net effect on fertility at BAU settings.

---

#### FM — Fecundity Multiplier (`fecundity_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 0, 0.2, 0.4, 0.6, 0.8, 0.9, 1.0, 1.05, 1.1 |
| **pyworld3 (FM)** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 0, 0.2, 0.4, 0.6, 0.8, 0.9, 1.0, 1.05, 1.1 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously capped at 0.87 — now allows FM > 1.0 at high LE per World3-03 specification.

---

#### CMPLE — Compensatory Multiplier from Perceived LE (`compensatory_fertility`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 3.0, 2.1, 1.6, 1.4, 1.3, 1.2, 1.1, 1.05, 1.0 |
| **pyworld3 (CMPLE)** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 3.0, 2.1, 1.6, 1.4, 1.3, 1.2, 1.1, 1.05, 1.0 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had a reduced range (1.40–0.90) and fewer breakpoints.

---

#### HSAPC — Health Services Allocations Per Capita (`health_services_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 250, 500, 750, 1000, 1250, 1500, 1750, 2000 | 0, 20, 50, 95, 140, 175, 200, 220, 230 |
| **pyworld3** | 0, 250, 500, 750, 1000, 1250, 1500, 1750, 2000 | 0, 20, 50, 95, 140, 175, 200, 220, 230 |

**Status: Aligned**

Replaced custom FSH fraction lookup with World3-03 HSAPC table. Maps service output per capita directly to health spending per capita.

---

### Capital / Industrial Sector

#### FIOACV — Consumption Fraction (`consumption_fraction`)

| | x | y |
|---|---|---|
| **Ours** | 0, 80, 160, 240, 320, 400, 480, 560, 640, 720, 800 | 0.3, 0.32, 0.34, 0.36, 0.38, 0.40, 0.44, 0.49, 0.55, 0.62, 0.70 |
| **pyworld3 (FIOACV)** | 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0 | 0.3, 0.32, 0.34, 0.36, 0.38, 0.43, 0.73, 0.77, 0.81, 0.82, 0.83 |

**Status: Intentional Deviation**

Differences:
- Different x-axis: ours uses absolute IOPC [$], pyworld3 uses IOPC/IOPCD ratio (IOPCD≈$400).
- **Capped at 0.70** (pyworld3 goes to 0.83). Real-world household consumption is ~55-60% of GDP.
- Smoothed above IOPC=400: pyworld3 has a cliff from 0.43→0.73 that traps IOPC growth.
- **Rationale:** The 0.70 cap prevents the consumption trap where >80% of output goes to consumption, starving industrial investment. The smooth curve avoids the discontinuity that causes IOPC stagnation.
- **Impact:** Moderate — allows IOPC to continue growing through mid-income range. Calibrated for historical IOPC RMSE <21% (REQ-026).

---

#### FIOAA — Fraction to Agriculture (`industrial_fraction_to_agriculture`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0, 2.5, 4.0 | 0.4, 0.22, 0.12, 0.04, 0.01, 0.005, 0.005 |
| **pyworld3 (FIOAA1)** | 0, 0.5, 1.0, 1.5, 2.0, 2.5 | 0.4, 0.2, 0.1, 0.025, 0.0, 0.0 |

**Status: Intentional Deviation**

Differences:
- Slightly higher values at moderate food_ratio: at x=0.5 ours=0.22 vs 0.20. At x=1.0 ours=0.12 vs 0.10.
- Floor of 0.005 at high food_ratio (x≥2.5) instead of zero — prevents oscillation in Stabilized preset.
- Extended x-range (up to 4.0 vs 2.5).
- **Input uses** `food_per_capita_smooth / IFPC(IOPC)` instead of pyworld3's `FPC / IFPC`. The smooth is an ODE stock preserved across RK4 stages.
- **Rationale:** The 0.005 floor prevents agricultural investment from dropping to zero, which caused yield collapse and oscillation in Technology/Stabilized scenarios. Slightly higher allocation compensates for LFH/PL food reduction factors.
- **Impact:** BAU nearly unchanged. Technology and Stabilized scenarios stable.

---

#### IFPC — Indicated Food Per Capita (`indicated_food_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 230, 480, 690, 850, 970, 1070, 1150, 1210, 1250 |
| **pyworld3 (IFPC1)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 230, 480, 690, 850, 970, 1070, 1150, 1210, 1250 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had lower values at moderate IOPC to compensate for missing LFH/PL factors.

---

#### FIOAS — Fraction to Services (`industrial_fraction_to_services`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0 | 0.30, 0.20, 0.10, 0.05, 0.0 |
| **pyworld3 (FIOAS1)** | 0, 0.5, 1.0, 1.5, 2.0 | 0.3, 0.2, 0.1, 0.05, 0.0 |

**Status: Exact Match**

---

#### ISOPC — Indicated Service Output Per Capita (`indicated_service_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 40, 300, 640, 1000, 1220, 1450, 1650, 1800, 2000 |
| **pyworld3 (ISOPC1)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 40, 300, 640, 1000, 1220, 1450, 1650, 1800, 2000 |

**Status: Exact Match**

Added March 2026 as part of dynamic ISOPC rework. Replaces hardcoded ISOPC=200. Allows service demand to scale with industrial development.

---

#### JPICU — Jobs per Industrial Capital Unit (`jobs_per_capital`)

| | x | y |
|---|---|---|
| **Ours** | 50, 200, 350, 500, 650, 800 | 0.00037, 0.00018, 0.00012, 0.00009, 0.00007, 0.00006 |
| **pyworld3 (JPICU)** | 50, 200, 350, 500, 650, 800 | 0.00037, 0.00018, 0.00012, 0.00009, 0.00007, 0.00006 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had inverted relationship (ascending instead of descending).

---

#### LFP — Labor Force Participation (`labor_force_participation`)

| | x | y |
|---|---|---|
| **Ours** | 0.5, 0.6, 0.7, 0.8 | 0.50, 0.55, 0.60, 0.65 |

**Status: Custom / No Direct Reference**

No direct equivalent in pyworld3. Custom table for display purposes.

---

### Agriculture Sector

#### LYMC — Land Yield Multiplier from Capital (`land_yield_multiplier_capital`)

| | x | y |
|---|---|---|
| **Ours** | 0, 40, 80, …, 1000 (26 points) | 1.0, 3.0, 3.8, 4.4, 4.9, 5.4, 5.7, 6.0, 6.3, 6.6, 6.9, 7.2, 7.4, 7.6, 7.8, 8.0, 8.2, 8.4, 8.6, 8.8, 9.0, 9.2, 9.4, 9.6, 9.8, 10.0 |
| **pyworld3 (LYMC)** | 0, 40, 80, …, 1000 (26 points) | 1.0, 3.0, 3.8, 4.4, 4.9, 5.4, 5.7, 6.0, 6.3, 6.6, 6.9, 7.2, 7.4, 7.6, 7.8, 8.0, 8.2, 8.4, 8.6, 8.8, 9.0, 9.2, 9.4, 9.6, 9.8, 10.0 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously truncated at x=400.

---

#### LYMAP — Land Yield Multiplier from Pollution (`land_yield_multiplier_pollution`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30 | 1.0, 1.0, 0.7, 0.4 |
| **pyworld3 (LYMAP1)** | 0, 10, 20, 30 | 1.0, 1.0, 0.7, 0.4 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had extended x-range with more gradual degradation.

---

#### LERD — Land Erosion Multiplier (`land_erosion_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0 | 0.0, 0.1, 0.3, 0.5, 0.7, 1.0, 1.5, 2.0, 2.5 |

**Status: Custom / No Direct Reference**

pyworld3 has LLMY (Land Life Multiplier from Yield) tables which serve a related but structurally different role.

---

#### LDCO — Land Development Cost (`land_development_cost`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 100, 117, 137, 161, 192, 232, 282, 344, 418, 507, 616 |

**Status: Custom / No Direct Reference**

pyworld3 has DCPH (Development Cost Per Hectare) indexed by PAL (absolute land area). Our exponential-growth cost curve uses fraction remaining with different parameterization.

---

#### UILPC — Urban-Industrial Land Per Capita (`urban_industrial_land_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 0.005, 0.008, 0.015, 0.025, 0.04, 0.055, 0.07, 0.08, 0.09 |
| **pyworld3 (UILPC)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 0.005, 0.008, 0.015, 0.025, 0.04, 0.055, 0.07, 0.08, 0.09 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had dramatically higher values (2-6× pyworld3).

---

#### LFDR — Land Fertility Degradation Rate (`land_fertility_degradation`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30 | 0.0, 0.1, 0.3, 0.5 |
| **pyworld3 (LFDR)** | 0, 10, 20, 30 | 0, 0.1, 0.3, 0.5 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had 60% lower degradation rates with extended x-range.

---

#### LFRT — Land Fertility Regeneration Time (`land_fertility_regeneration_time`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.02, 0.04, 0.06, 0.08, 0.10 | 20, 13, 8, 4, 2, 2 |
| **pyworld3 (LFRT)** | 0, 0.02, 0.04, 0.06, 0.08, 0.10 | 20, 13, 8, 4, 2, 2 |

**Status: Exact Match**

---

#### FALM — Fraction Allocated to Land Maintenance (`fraction_land_maintenance`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4 | 0.0, 0.04, 0.07, 0.09, 0.10 |
| **pyworld3 (FALM)** | 0, 1, 2, 3, 4 | 0.0, 0.04, 0.07, 0.09, 0.10 |

**Status: Exact Match**

---

#### CMPLE — Compensatory Fertility (see Population Sector above)

Listed under Population sector.

---

#### FRNF — Food Fertility Multiplier (`food_fertility_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0 | 0.0, 0.6, 1.0, 1.05, 1.1 |

**Status: Custom / No Direct Reference**

pyworld3 has FCE (Food Consumption Effect) tables with different functional roles.

---

### Resource Sector

#### FCAOR — Fraction of Capital to Resource Extraction (`capital_fraction_resource_extraction`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.7, 0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.05 |
| **pyworld3 (FCAOR1)** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.7, 0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.05 |

**Status: Exact Match**

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had smoother, higher-allocation curve with linear decline to zero.

---

### Pollution Sector

#### PPGIO — Pollution Generation from Industry (`pollution_generation_industry`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4, 5 | 0.0, 1.0, 1.5, 1.9, 2.16, 2.36 |

**Status: Custom / No Direct Reference**

pyworld3 uses constant PPGIO multiplier (not a table). Our table implements diminishing pollution intensity as industrial output grows.

---

#### PPGAO — Pollution Generation from Agriculture (`pollution_generation_agriculture`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4 | 0.0, 1.0, 1.7, 2.2, 2.5 |

**Status: Custom / No Direct Reference**

Similar to PPGIO — pyworld3 uses constant multiplier. Our table captures diminishing marginal pollution from agricultural intensification.

---

#### PPASR — Pollution Assimilation Time (`pollution_assimilation_time`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2.5, 5, 10, 25, 50, 100 | 1.5, 2.5, 5.0, 10.0, 20.0, 40.0, 80.0, 160.0 |
| **pyworld3 (AHLM)** | 1, 251, 501, 751, 1001 | 1, 11, 21, 31, 41 |

**Status: Intentional Deviation (structural)**

Completely different functional form. pyworld3 AHLM is the Assimilation Half-Life Multiplier (dimensionless, linear), applied to a base half-life. Ours is the assimilation time directly in years. Our x-axis operates on pollution index 0–100; pyworld3 on 1–1001. Our model computes pollution assimilation directly (pollution / assimilation_time), while World3-03 uses half-life × multiplier.

---

## Missing pyworld3 Tables (not in our model)

| pyworld3 Table | Purpose | Why Missing |
|---|---|---|
| LMHS1, FIOAA2, LYMAP2, FIOAS2, ISOPC2, IFPC2, FCAOR2 | Policy-switch variants (pre/post-intervention) | Not yet implemented; using LMHS2, single tables for others |
| FIALD | Fraction to land development | Different land allocation mechanism |
| MLYMC | Marginal land yield from capital | Not modeled separately |
| LLMY1, LLMY2 | Land life multiplier from yield | Replaced by LERD (custom erosion) |
| DCPH | Development cost per hectare | Replaced by LDCO (custom cost curve) |
| PCRUM | Per capita resource usage | Implicit in resource depletion equation |
| CUF | Capital utilization fraction | Not modeled (assumed CUF=1) |
| JPSCU, JPH | Service/agriculture jobs | Not used in core dynamics |
| SFSN | Social fertility norm | Merged into DCFS lookup |
| FSAFC | Family size adjustment from consumption | Not modeled |
| FCE_TOCLIP | Food consumption effect | Replaced by FRNF (custom) |

---

## Recommendations for Future Work

1. **Policy-switch tables**: World3-03 has paired tables (FIOAA1/FIOAA2, LYMAP1/LYMAP2, etc.) for pre/post policy switch. We use single tables. Consider adding policy-switch variants for Technology/Stabilized scenarios.

2. **LFH/PL factors**: Our model includes LFH=0.7 and PL=0.1 in the food equation. Lookup tables are aligned to pyworld3 reference values.

3. **DCFS sensitivity**: The calibrated DCFS values are the most significant deviation from pyworld3. Monitor during future tuning — if perceived-LE dynamics change, DCFS may need recalibration.

4. **PPASR reconciliation**: Our pollution assimilation mechanism differs structurally from pyworld3. Consider implementing the half-life multiplier approach for better alignment.

5. **Nebel et al. fine-tuning**: BAU parameters (alic1=13yr, tech_rate=0.014) are tuned per Nebel et al. (2024) methodology — adjusting high-sensitivity parameters (industrial capital lifetime, technology growth) within plausible ranges to match historical data through 2023. Future recalibration should follow the same methodology when new empirical data becomes available.

---

## How to Re-run This Audit

Use the `/audit-tables` slash command in Claude Code:
```
/audit-tables
```

This will re-fetch the pyworld3 reference, compare against current `tables.rs`, and update this document.
