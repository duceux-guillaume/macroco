---
description: Audit lookup tables against World3-03 reference (pyworld3)
---

# Lookup Table Audit

Compare our World3 lookup tables against the pyworld3 reference implementation (faithfully digitized from the World3-03 Vensim model, Meadows 2004).

## Steps

1. **Read our current tables:**
   Read `crates/world3-core/src/lookup/tables.rs` and extract every `LookupTable::new(...)` call — record the field name, x-values, and y-values.

2. **Fetch the pyworld3 reference:**
   Use WebFetch to get the raw JSON from:
   `https://raw.githubusercontent.com/cvanwynsberghe/pyworld3/master/pyworld3/functions_table_world3.json`
   Extract all table names, x-values, and y-values.

3. **Compare each table using this name mapping:**

   | Our field name | pyworld3 table(s) | Notes |
   |---|---|---|
   | `mortality_0_14` | M1 | |
   | `mortality_15_44` | M2 | |
   | `mortality_45_64` | M3 | |
   | `mortality_65_plus` | M4 | |
   | `life_exp_multiplier_food` | LMF | Different x-resolution |
   | `life_exp_multiplier_health` | LMHS1, LMHS2 | We use single table |
   | `life_exp_multiplier_crowding` | CMI × FPU | Structural simplification |
   | `life_exp_multiplier_pollution` | LMP | |
   | `desired_family_size` | SFSN (× dcfsn=3.8) | Merged into single table |
   | `family_planning_multiplier` | FRSN | Different x-axis |
   | `fecundity_multiplier` | FM | |
   | `compensatory_fertility` | CMPLE | Different scale |
   | `fraction_services_health` | HSAPC (indirect) | Structural difference |
   | `consumption_fraction` | FIOACV | Absolute vs ratio x-axis |
   | `industrial_fraction_to_agriculture` | FIOAA1, FIOAA2 | |
   | `industrial_fraction_to_services` | FIOAS1, FIOAS2 | |
   | `jobs_per_capital` | JPICU | Structural difference |
   | `labor_force_participation` | — | Custom table |
   | `land_yield_multiplier_capital` | LYMC | Truncated x-range |
   | `land_yield_multiplier_pollution` | LYMAP1, LYMAP2 | Extended, gentler |
   | `land_erosion_multiplier` | LLMY1, LLMY2 (related) | Custom formulation |
   | `land_development_cost` | DCPH (related) | Different parameterization |
   | `urban_industrial_land_per_capita` | UILPC | Higher values |
   | `land_fertility_degradation` | LFDR | Lower rates |
   | `land_fertility_regeneration_time` | LFRT | |
   | `fraction_land_maintenance` | FALM | |
   | `food_fertility_multiplier` | FCE (related) | Custom |
   | `capital_fraction_resource_extraction` | FCAOR1, FCAOR2 | Different curve shape |
   | `pollution_generation_industry` | — | Custom (pyworld3 uses constant) |
   | `pollution_generation_agriculture` | — | Custom (pyworld3 uses constant) |
   | `pollution_assimilation_time` | AHLM | Structural difference |

4. **For each table, determine status:**
   - **Exact Match**: x-values and y-values identical (within floating-point tolerance).
   - **Intentional Deviation**: Values differ but there is a documented rationale in `tables.rs` comments or `docs/audit.md`. Note the magnitude of deviation at key calibration points.
   - **Significant Deviation**: Values differ with no clear rationale — flag for review.
   - **New / Unaudited**: Table exists in our code but is not covered by the mapping above. Flag for investigation.

5. **Check for new tables:**
   Compare the list of `pub` fields in `WorldLookupTables` struct against the mapping above. Any field not in the mapping is new and needs audit.

6. **Check for missing tables:**
   List pyworld3 tables that have no mapping in our code. Note which are structurally unnecessary vs potentially missing.

7. **Update `docs/audit.md`:**
   Overwrite `docs/audit.md` with the full audit results, preserving the existing format:
   - Header with date, sources
   - Summary table (counts by category)
   - Table-by-table comparison with x/y values, status, rationale, impact
   - Recommendations section
   - Update the date to the current session date
