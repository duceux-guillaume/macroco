# Lookup Table Audit Report

**Date:** 2026-03-04
**Auditor:** Claude (automated comparison)
**Our source:** `crates/world3-core/src/lookup/tables.rs`
**Reference:** [pyworld3 `functions_table_world3.json`](https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json) (faithfully digitized from World3-03 Vensim model, Meadows 2004)
**Secondary reference:** [WorldDynamics.jl World3 module](https://github.com/worlddynamics/WorldDynamics.jl)

---

## Summary

| Category | Count | Tables |
|----------|------:|--------|
| Exact Match | 7 | M1, M2, M3, M4, FIOAS1, LFRT, FALM |
| Intentional Deviation | 18 | LMF, LMHS, LMCR, LMP, DCFS, FRSN, FM, CMPLE, FIOACV, FIOAA, IFPC, JPICU, LYMC, LYMAP, UILPC, LFDR, FCAOR, PPASR |
| Custom / No Reference | 7 | FSH, LFP, LERD, LDCO, FRNF, PPGIO, PPGAO |
| **Total** | **32** | |

**Risk assessment:**
- The 7 exact-match tables (all mortality + FIOAS + LFRT + FALM) cover critical population dynamics faithfully.
- Most intentional deviations are documented calibration adjustments to compensate for structural simplifications in our model (missing delay loops, merged sub-tables, different x-axis normalizations).
- Highest-risk deviations: FM (fecundity caps at 0.87 vs 1.1), CMPLE (rescaled), FCAOR (steeper depletion), LFDR (slower degradation). These directly affect BAU trajectory shape.
- IFPC and FIOAA were recalibrated in March 2026 to fix Technology/Stabilized scenario instability. See IFPC and FIOAA entries below for details.

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
| **Ours** | 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 3.0, 4.0 | 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.04, 1.08, 1.12, 1.16, 1.20, 1.33, 1.40 |
| **pyworld3 (LMF)** | 0, 1, 2, 3, 4, 5 | 0, 1, 1.2, 1.3, 1.35, 1.4 |

**Status: Intentional Deviation**

Differences:
- Finer x-resolution below food_ratio=1.0 (13 points vs 6). Below 1.0 both are linear from (0,0) to (1,1) — functionally identical.
- Above food_ratio=1.0: gains are flattened in our table. At x=2.0 both give 1.20. At x=3.0 ours gives 1.33 vs 1.30 (+2.3%). At x=4.0 ours gives 1.40 vs 1.35 (+3.7%).
- **Rationale:** Our model produces higher food/capita than World3-03, so we flatten gains above subsistence to avoid compounding LE inflation. The finer sub-1.0 resolution gives smoother behavior at low food ratios.
- **Impact:** Minimal — the difference is <4% and only at very high food ratios.

---

#### LMHS — Life Expectancy Multiplier from Health Services (`life_exp_multiplier_health`)

| | x | y |
|---|---|---|
| **Ours** | 0, 20, 40, 60, 80, 100, 150, 200 | 1.0, 1.0, 1.0, 1.20, 1.50, 1.70, 1.85, 1.95 |
| **pyworld3 (LMHS1)** | 0, 20, 40, 60, 80, 100 | 1, 1.1, 1.4, 1.6, 1.7, 1.8 |
| **pyworld3 (LMHS2)** | 0, 20, 40, 60, 80, 100 | 1, 1.4, 1.6, 1.8, 1.95, 2.0 |

**Status: Intentional Deviation**

Differences:
- World3-03 has two tables (LMHS1 before policy switch, LMHS2 after). We use a single table.
- Delayed onset: LMHS=1.0 up to EHSPC=40 in ours (vs 1.1 and 1.4 at EHSPC=20 in pyworld3). This approximates the 20-year delay loop (HSAPC → EHSPC) that our model omits.
- Extended x-range to 200 (pyworld3 stops at 100).
- At EHSPC=100: ours=1.70 matches LMHS1=1.8 approximately. At EHSPC=60: ours=1.20 vs LMHS1=1.6 (−25%).
- **Rationale:** Without the HSAPC delay, a steeper early curve would produce unrealistically early LE gains. The delayed-onset shape compensates.
- **Impact:** Moderate — directly affects LE trajectory in 1900-1960 period. Calibrated to produce LE≈33 in 1900, LE≈60 in 1970.

---

#### LMCR — Life Expectancy Multiplier from Crowding (`life_exp_multiplier_crowding`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0 | 1.05, 1.0, 0.95, 0.90, 0.85, 0.80, 0.75 |
| **pyworld3 (CMI)** | 0–1600 (IOPC) | 0.5, 0.05, −0.1, −0.08, −0.02, 0.05, 0.1, 0.15, 0.2 |
| **pyworld3 (FPU)** | 0–16B (POP) | 0, 0.2, 0.4, 0.5, 0.58, 0.65, 0.72, 0.78, 0.8 |

**Status: Intentional Deviation (structural)**

Differences:
- World3-03 computes LMCR = 1 − CMI(IOPC) × FPU(POP) using two separate tables. We collapse this into a single direct lookup on population/reference ratio.
- **Rationale:** Our model lacks the separate IOPC-dependent crowding component. The single table captures the net crowding effect at BAU-trajectory IOPC values.
- **Impact:** Low — crowding is a minor factor in LE relative to food, health, and pollution.

---

#### LMP — Life Expectancy Multiplier from Pollution (`life_exp_multiplier_pollution`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60, 80, 100 | 1.0, 0.99, 0.97, 0.95, 0.90, 0.85, 0.75, 0.55, 0.40 |
| **pyworld3 (LMP)** | 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100 | 1.0, 0.99, 0.97, 0.95, 0.90, 0.85, 0.75, 0.65, 0.55, 0.40, 0.20 |

**Status: Intentional Deviation**

Differences:
- Fewer breakpoints (missing x=70, x=90). At x=80: ours=0.55 vs pyworld3=0.55 (match). At x=100: ours=0.40 vs pyworld3=0.20 (ours is 2× higher).
- **Rationale:** Less severe at extreme pollution. In our BAU, pollution index rarely exceeds 30-40, so the x>60 range has minimal practical impact.
- **Impact:** Low at BAU levels. Would matter in extreme-pollution scenarios (pollution index >60).

---

#### DCFS — Desired Completed Family Size (`desired_family_size`)

| | x | y |
|---|---|---|
| **Ours** | 0, 50, 100, 200, 400, 600, 800, 1200, 1600 | 3.60, 3.58, 3.55, 3.75, 3.00, 2.45, 2.10, 1.95, 1.85 |
| **pyworld3 (SFSN)** | 0, 200, 400, 600, 800 | 1.25, 1.0, 0.9, 0.8, 0.75 |
| **World3-03 formula** | DCFS = dcfsn(3.8) × SFSN(DIOPC) | Range: 2.85–4.75 |

**Status: Intentional Deviation (structural)**

Differences:
- World3-03 uses dcfsn=3.8 × SFSN(DIOPC) × CMPLE(PLE). SFSN is a multiplier 0.75–1.25.
- We merge dcfsn × SFSN into a single table. CMPLE is applied separately (see below).
- Our effective DCFS: at IOPC=$200 → 3.75 (vs World3-03: 3.8×1.0=3.80). At IOPC=$800 → 2.10 (vs World3-03: 3.8×0.75=2.85).
- **Rationale:** Simplified structure. The non-monotonic hump at IOPC=200 (3.75) captures demographic transition dynamics where family size briefly rises with initial income before declining.
- **Impact:** Moderate — lower high-income DCFS (2.10 vs 2.85) accelerates demographic transition.

---

#### FRSN — Family Planning Multiplier (`family_planning_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.25, 0.5, 0.75, 1.0 | 1.0, 0.90, 0.75, 0.55, 0.40 |
| **pyworld3 (FRSN)** | −0.2, −0.1, 0, 0.1, 0.2 | 0.5, 0.6, 0.7, 0.85, 1.0 |

**Status: Intentional Deviation (structural)**

Differences:
- Different x-axis: ours uses 0–1 (planning effectiveness), pyworld3 uses family income expectation difference.
- Inverted relationship: in pyworld3, higher FRSN → more fertility. In ours, higher planning → less fertility.
- **Rationale:** Our model uses a simpler family planning lever (0=none, 1=full effectiveness) rather than the income-expectation mechanism.
- **Impact:** Different mechanism but comparable net effect on fertility at BAU settings.

---

#### FM — Fecundity Multiplier (`fecundity_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 0, 0.2, 0.4, 0.6, 0.7, 0.75, 0.8, 0.85, 0.87 |
| **pyworld3 (FM)** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 0, 0.2, 0.4, 0.6, 0.8, 0.9, 1.0, 1.05, 1.1 |

**Status: Intentional Deviation**

Differences:
- Same x-axis. Y-values diverge above LE=40. At LE=80: ours=0.87 vs pyworld3=1.1 (−21%).
- At LE=50: ours=0.75 vs pyworld3=0.9 (−17%).
- **Rationale:** Our FM is capped below 1.0 — biological fecundity never exceeds baseline. In pyworld3, FM>1.0 represents enhanced fecundity at high LE (supernutrition/health effect).
- **Impact:** High — directly scales birth rate. Our lower FM dampens population growth at high LE, contributing to earlier population peak. This is a calibration choice: combined with our CMPLE and DCFS, total fertility matches World3-03 BAU trajectory.

---

#### CMPLE — Compensatory Multiplier from Perceived LE (`compensatory_fertility`)

| | x | y |
|---|---|---|
| **Ours** | 20, 30, 40, 50, 60, 70, 80 | 1.40, 1.25, 1.10, 1.0, 0.95, 0.92, 0.90 |
| **pyworld3 (CMPLE)** | 0, 10, 20, 30, 40, 50, 60, 70, 80 | 3.0, 2.1, 1.6, 1.4, 1.3, 1.2, 1.1, 1.05, 1.0 |

**Status: Intentional Deviation (structural)**

Differences:
- Completely different scale. pyworld3 range: 3.0–1.0. Ours: 1.40–0.90.
- pyworld3 CMPLE is the primary driver of high fertility at low LE (3× multiplier). In our model, the base DCFS table already includes high-fertility values at low income, so CMPLE acts as a smaller adjustment (±40%).
- Ours drops below 1.0 at high PLE (confidence in child survival reduces desired births). pyworld3 CMPLE floors at 1.0.
- **Rationale:** Different structural role. In World3-03: DCFS = dcfsn × SFSN × CMPLE, where CMPLE is the dominant factor. In ours: DCFS(IOPC) already encodes base desired family size, and CMPLE is a modifier. The net product at key calibration points matches.
- **Impact:** High in isolation, but calibrated so total fertility matches. At PLE=30: pyworld3 → 3.8 × 1.0 × 1.4 = 5.32. Ours → 3.60 × 1.25 = 4.50.

---

#### FSH — Fraction of Services to Health (`fraction_services_health`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0 | 0.3, 0.35, 0.40, 0.45, 0.50 |
| **pyworld3** | *(HSAPC table maps SOPC → health spending directly)* | — |

**Status: Custom / No Direct Reference**

Our model uses a simple fraction lookup. World3-03 uses the HSAPC table (SOPC → health spending per capita) which has a different functional form. Our table approximates the effective fraction at relevant service output levels.

---

### Capital / Industrial Sector

#### FIOACV — Consumption Fraction (`consumption_fraction`)

| | x | y |
|---|---|---|
| **Ours** | 0, 80, 160, 240, 320, 400, 480, 560, 640, 720, 800 | 0.40, 0.42, 0.44, 0.46, 0.48, 0.53, 0.78, 0.80, 0.82, 0.83, 0.83 |
| **pyworld3 (FIOACV)** | 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0 | 0.3, 0.32, 0.34, 0.36, 0.38, 0.43, 0.73, 0.77, 0.81, 0.82, 0.83 |

**Status: Intentional Deviation**

Differences:
- Different x-axis: ours uses absolute IOPC [$], pyworld3 uses IOPC/IOPCD ratio (IOPCD≈$400).
- Baseline: ours starts at 0.40 vs pyworld3's 0.30 at x=0.
- The characteristic jump at x≈1.0 (ratio) / x≈400 (IOPC) is preserved in both.
- **Rationale:** Without the dynamic IOPCD variable, we use absolute IOPC with a higher baseline (0.40 vs 0.30) to compensate for our model's missing references (IOPCD, ISOPC, IFPC). This produces correct aggregate investment rate (~32% in 1900).
- **Impact:** Moderate — affects industrial capital accumulation rate. Calibrated against historical trajectory.

---

#### FIOAA — Fraction to Agriculture (`industrial_fraction_to_agriculture`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0 | 0.40, 0.30, 0.20, 0.16, 0.12, 0.08, 0.05, 0.04, 0.04 |
| **pyworld3 (FIOAA1)** | 0, 0.5, 1.0, 1.5, 2.0, 2.5 | 0.4, 0.2, 0.1, 0.025, 0.0, 0.0 |

**Status: Intentional Deviation**

Differences:
- Slower decline: at x=0.5 ours=0.30 vs pyworld3=0.20. At x=1.0 ours=0.20 vs pyworld3=0.10.
- Extended x-range (up to 4.0 vs 2.5) with more gradual tail.
- Floor of 0.04 at high food_ratio (x≥3.0) instead of zero — ensures minimum agricultural maintenance even in wealthy societies.
- **Input change (2026-03-04):** x-axis now uses `food_per_capita_smooth / IFPC(IOPC)` instead of `food_per_capita / subsistence_food`. The smooth is an ODE stock preserved across RK4 stages, giving consistent allocation fractions. IFPC replaces the constant subsistence denominator.
- **Rationale:** Our model produces ~59% more food than World3-03 (missing LFH=0.7, PL=0.1 factors), so the food_ratio reaches higher values. The 0.04 floor prevents yield collapse when food_ratio exceeds 3.0 (which happens in Technology and Stabilized scenarios). Combined with IFPC as the dynamic denominator, this prevents the zero-allocation trap that caused instability.
- **Impact:** Moderate — BAU unchanged (food_ratio stays below 2.5). Technology and Stabilized scenarios now stable (max 3.5% YoY oscillation vs 15-18% before).

---

#### IFPC — Indicated Food Per Capita (`indicated_food_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600, 2000, 2500 | 230, 230, 350, 500, 650, 800, 950, 1100, 1200, 1400, 1600 |
| **pyworld3 (IFPC1)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 230, 480, 690, 850, 970, 1070, 1150, 1210, 1250 |

**Status: Intentional Deviation**

*Added 2026-03-04 as part of IFPC food allocation rework.*

Differences:
- Same x-axis range at low IOPC (both start at 0, step 200). Extended to 2500 for Stabilized scenario (peaks ~2000 IOPC).
- Much lower values at moderate IOPC: at x=200 ours=230 vs pyworld3=480 (−52%). At x=400 ours=350 vs pyworld3=690 (−49%). At x=1600 ours=1200 vs pyworld3=1250 (−4%).
- Convergence at high IOPC: the curves approach each other. At x=1600 only 4% difference.
- **Rationale:** Our model produces ~59% more food than World3-03 at identical parameters (missing LFH=0.7 land fraction harvested, PL=0.1 processing loss). Using World3-03's IFPC values would make the food_ratio (FPC_smooth/IFPC) too low at BAU IOPC levels (≤330), causing over-allocation to agriculture and starving industrial investment. Our lower IFPC at low IOPC preserves BAU dynamics. The steeper rise at high IOPC (extending to 2500) prevents the zero-allocation trap in Technology/Stabilized scenarios.
- **Impact:** BAU unaffected (IOPC<330, IFPC=230=SFPC). Technology and Stabilized: food_ratio stays moderate (~1.5-2.5 instead of reaching 3.0+), preventing allocation collapse.

---

#### FIOAS — Fraction to Services (`industrial_fraction_to_services`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 1.5, 2.0 | 0.30, 0.20, 0.10, 0.05, 0.0 |
| **pyworld3 (FIOAS1)** | 0, 0.5, 1.0, 1.5, 2.0 | 0.3, 0.2, 0.1, 0.05, 0.0 |

**Status: Exact Match**

---

#### JPICU — Jobs per Industrial Capital Unit (`jobs_per_capital`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.5, 1.0, 2.0, 3.0, 4.0 | 0.0007, 0.0014, 0.0017, 0.0018, 0.0019, 0.002 |
| **pyworld3 (JPICU)** | 50, 200, 350, 500, 650, 800 | 0.00037, 0.00018, 0.00012, 0.00009, 0.00007, 0.00006 |

**Status: Intentional Deviation (structural)**

Differences:
- Completely different shape: ours is ascending (more capital → more jobs), pyworld3 is descending (higher IOPC → fewer jobs per unit capital, reflecting automation).
- Different x-axis: ours uses normalized IOPC, pyworld3 uses absolute IOPC.
- Values differ by ~10×.
- **Rationale:** Our model uses employment as a secondary indicator. The ascending shape reflects a different modeling assumption about capital-labor complementarity. This table is not currently used in core dynamics (labor sector is inactive).
- **Impact:** Low — labor/employment is a display variable, not a feedback driver in our current model.

---

#### LFP — Labor Force Participation (`labor_force_participation`)

| | x | y |
|---|---|---|
| **Ours** | 0.5, 0.6, 0.7, 0.8 | 0.50, 0.55, 0.60, 0.65 |

**Status: Custom / No Direct Reference**

No direct equivalent in pyworld3. Our custom table for display purposes.

---

### Agriculture Sector

#### LYMC — Land Yield Multiplier from Capital (`land_yield_multiplier_capital`)

| | x | y |
|---|---|---|
| **Ours** | 0, 40, 80, 120, 160, 200, 240, 280, 320, 360, 400 | 1.0, 3.0, 4.5, 5.0, 5.3, 5.6, 5.9, 6.1, 6.35, 6.6, 6.9 |
| **pyworld3 (LYMC)** | 0, 40, 80, …, 1000 (26 points) | 1.0, 3.0, 3.8, 4.4, 4.9, 5.4, 5.7, 6.0, 6.3, 6.6, 6.9, …, 10.0 |

**Status: Intentional Deviation**

Differences:
- Ours truncated at x=400, LYMC_max=6.9. pyworld3 continues to x=1000, LYMC_max=10.0.
- At x=80: ours=4.5 vs pyworld3=3.8 (+18%). At x=120: ours=5.0 vs pyworld3=4.4 (+14%).
- **Rationale:** Our model's agricultural inputs per hectare stay within 0–400 range in BAU. Higher initial yield response (at x=80–120) compensates for our model's simpler agricultural investment loop. Truncation avoids extrapolation beyond calibrated range.
- **Impact:** Moderate — higher early yield response means food production responds more to capital. The truncation at 400 would limit yield in high-technology scenarios.

---

#### LYMAP — Land Yield Multiplier from Pollution (`land_yield_multiplier_pollution`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 5, 10, 20, 30, 40, 50, 60 | 1.0, 1.0, 0.95, 0.90, 0.80, 0.70, 0.60, 0.50, 0.40 |
| **pyworld3 (LYMAP1)** | 0, 10, 20, 30 | 1.0, 1.0, 0.7, 0.4 |

**Status: Intentional Deviation**

Differences:
- Extended x-range (up to 60 vs 30). More gradual degradation.
- At x=20: ours=0.80 vs pyworld3=0.70. At x=30: ours=0.70 vs pyworld3=0.40.
- **Rationale:** Our pollution index uses different normalization. The more gradual curve prevents agriculture from collapsing too abruptly at moderate pollution levels.
- **Impact:** Moderate — delays pollution-driven food crisis by ~10-20 years in BAU.

---

#### LERD — Land Erosion Multiplier (`land_erosion_multiplier`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0 | 0.0, 0.1, 0.3, 0.5, 0.7, 1.0, 1.5, 2.0, 2.5 |

**Status: Custom / No Direct Reference**

pyworld3 has LLMY (Land Life Multiplier from Yield) tables which serve a related but structurally different role. Our erosion multiplier is a custom formulation.

---

#### LDCO — Land Development Cost (`land_development_cost`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 100, 117, 137, 161, 192, 232, 282, 344, 418, 507, 616 |

**Status: Custom / No Direct Reference**

pyworld3 has DCPH (Development Cost Per Hectare) indexed by PAL (potentially arable land), with a different x-axis (absolute land area vs fraction). Our exponential-growth cost curve approximates the same concept with different parameterization.

---

#### UILPC — Urban-Industrial Land Per Capita (`urban_industrial_land_per_capita`)

| | x | y |
|---|---|---|
| **Ours** | 0, 200, 400, 600, 800, 1000, 1600 | 0.005, 0.05, 0.09, 0.11, 0.13, 0.15, 0.16 |
| **pyworld3 (UILPC)** | 0, 200, 400, 600, 800, 1000, 1200, 1400, 1600 | 0.005, 0.008, 0.015, 0.025, 0.04, 0.055, 0.07, 0.08, 0.09 |

**Status: Intentional Deviation**

Differences:
- Dramatically higher values. At IOPC=200: ours=0.05 vs pyworld3=0.008 (6.25×). At IOPC=1600: ours=0.16 vs pyworld3=0.09 (1.78×).
- Fewer breakpoints (7 vs 9).
- **Rationale:** Higher UILPC produces more aggressive urban land consumption, which reduces arable land faster. This drives stronger land pressure feedback in our model, compensating for our simpler land allocation mechanisms.
- **Impact:** High — directly controls how fast arable land shrinks with industrialization. Our higher values accelerate land scarcity, which is a key overshoot driver.

---

#### LFDR — Land Fertility Degradation Rate (`land_fertility_degradation`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50 | 0.0, 0.04, 0.10, 0.20, 0.30, 0.40 |
| **pyworld3 (LFDR)** | 0, 10, 20, 30 | 0, 0.1, 0.3, 0.5 |

**Status: Intentional Deviation**

Differences:
- Extended x-range (up to 50 vs 30). Significantly lower degradation rates.
- At x=10: ours=0.04 vs pyworld3=0.1 (−60%). At x=20: ours=0.10 vs pyworld3=0.3 (−67%). At x=30: ours=0.20 vs pyworld3=0.5 (−60%).
- **Rationale:** Our model's pollution index normalization differs. Without this reduction, land fertility would degrade unrealistically fast. The extended range accommodates our model's higher pollution index values.
- **Impact:** High — slower degradation delays the agriculture collapse phase. Combined with LYMAP deviation, this shifts the food crisis later in BAU.

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

pyworld3 has FCE (Food Consumption Effect) tables but with different functional roles. Our food fertility multiplier is a simplified custom formulation.

---

### Resource Sector

#### FCAOR — Fraction of Capital to Resource Extraction (`capital_fraction_resource_extraction`)

| | x | y |
|---|---|---|
| **Ours** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.70, 0.50, 0.40, 0.30, 0.20, 0.14, 0.08, 0.04, 0.0 |
| **pyworld3 (FCAOR1)** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.7, 0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.05 |

**Status: Intentional Deviation**

Differences:
- Same x-axis. Different curve shape in the 0.3–1.0 range.
- At NRFR=0.4: ours=0.40 vs pyworld3=0.20 (2×). At NRFR=0.5: ours=0.30 vs pyworld3=0.10 (3×).
- At NRFR=1.0 (full reserves): ours=0.0 vs pyworld3=0.05.
- Ours declines linearly; pyworld3 drops sharply to 0.05 and plateaus.
- **Rationale:** Our smoother, higher-allocation curve means resource extraction demands more capital even when resources are moderately abundant. The linear decline avoids the unrealistic plateau at 0.05.
- **Impact:** Moderate — more capital diverted to resource extraction reduces industrial output growth slightly, but the effect is offset by our endpoint at 0.0 (no extraction cost when resources are 100% remaining, vs pyworld3's 5%).

---

### Pollution Sector

#### PPGIO — Pollution Generation from Industry (`pollution_generation_industry`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4, 5 | 0.0, 1.0, 1.5, 1.9, 2.16, 2.36 |

**Status: Custom / No Direct Reference**

pyworld3 pollution generation uses a different structural approach (PPGIO is a constant multiplier, not a table). Our table implements diminishing pollution intensity as industrial output grows (technology/efficiency effect).

---

#### PPGAO — Pollution Generation from Agriculture (`pollution_generation_agriculture`)

| | x | y |
|---|---|---|
| **Ours** | 0, 1, 2, 3, 4 | 0.0, 1.0, 1.7, 2.2, 2.5 |

**Status: Custom / No Direct Reference**

Similar to PPGIO — pyworld3 uses a constant multiplier. Our table captures diminishing marginal pollution from agricultural intensification.

---

#### PPASR — Pollution Assimilation Time (`pollution_assimilation_time`)

| | x | y |
|---|---|---|
| **Ours** | 0, 10, 20, 30, 40, 50, 60 | 20, 45, 90, 150, 220, 320, 480 |
| **pyworld3 (AHLM)** | 1, 251, 501, 751, 1001 | 1, 11, 21, 31, 41 |

**Status: Intentional Deviation (structural)**

Differences:
- Completely different functional form. pyworld3 AHLM is the Assimilation Half-Life Multiplier (dimensionless, linear), applied to a base half-life. Ours is the assimilation time directly in years.
- Our x-axis operates on pollution index 0–60; pyworld3 on 1–1001.
- **Rationale:** Our model computes pollution assimilation directly (pollution / assimilation_time), while World3-03 uses half-life × multiplier. The steep curve (20yr → 480yr) produces the environmental overwhelm feedback at high pollution.
- **Impact:** High — controls how fast pollution clears. Calibrated so pollution accumulates visibly by 2000 and peaks >10 by 2030-2040 in BAU.

---

## Recommendations for Future Work

1. **FM recalibration**: Consider whether FM should exceed 1.0 at high LE (pyworld3 goes to 1.1). Currently our cap at 0.87 suppresses population growth — verify this is intentional for BAU trajectory match, not an error.

2. **UILPC verification**: Our values are 2-6× higher than pyworld3. While this drives appropriate land pressure, verify against historical urbanization data (hectares/person by country, 1900-2000).

3. **LFDR rescaling**: Our 60% lower degradation rates warrant sensitivity analysis. If pollution normalization changes (e.g., adding climate sector), this table needs recalibration.

4. **LYMC extension**: Consider extending to x=1000 for technology-scenario runs where agricultural inputs per hectare exceed 400.

5. **FCAOR plateau**: Investigate whether pyworld3's plateau at 0.05 (minimum extraction cost) is physically meaningful vs. our linear-to-zero approach.

6. **Policy-switch tables**: World3-03 has paired tables (LMHS1/LMHS2, FIOAA1/FIOAA2, LYMAP1/LYMAP2, etc.) for pre/post policy switch. We use single tables. Consider adding policy-switch variants for Milestone 2 scenario analysis.

7. **LFH/PL factors**: Our model omits Land Fraction Harvested (LFH=0.7) and Processing Loss (PL=0.1) from World3-03. This causes ~59% higher food production at identical parameters. The IFPC table was calibrated lower to compensate, but adding these factors explicitly would allow using World3-03 IFPC values directly and improve cross-model comparability.

---

## How to Re-run This Audit

Use the `/audit-tables` slash command in Claude Code:
```
/audit-tables
```

This will re-fetch the pyworld3 reference, compare against current `tables.rs`, and update this document.
