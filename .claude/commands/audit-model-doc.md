---
description: Audit model documentation against source code and World3-03 reference
---

# Model Documentation Audit

Verify that `docs/model/` is complete, correctly templated, and in sync with the Rust source code and the pyworld3 World3-03 reference.

## Arguments

- No arguments: full audit, report issues, offer to fix.
- `--check`: read-only, exit with error message on any issue (for CI).
- `--diff`: only audit files corresponding to code changed in current worktree vs main. Fast mode for pre-PR.

## Phase 1 — Completeness

1. Read `crates/world3-core/src/lookup/tables.rs`. Extract every `pub` field name from the `WorldLookupTables` struct.
2. For each field, check that `docs/model/tables/<kebab-name>.md` exists. (Naming: `desired_family_size` → `desired-family-size.md`)
3. Read `crates/world3-core/src/model/params.rs`. Extract every field from `ScenarioParams` (excluding `meta`).
4. For each field, check that `docs/model/parameters/<kebab-name>.md` exists.
5. Check that each `.rs` file in `crates/world3-core/src/model/sectors/` (excluding `mod.rs`) has a matching `docs/model/sectors/<name>.md`.
6. Check for orphans: any `.md` file in `docs/model/tables/`, `docs/model/parameters/`, or `docs/model/sectors/` that does not correspond to a code entity.
7. Report missing files and orphans.

## Phase 2 — Template Conformance

For each file, check required headings exist:

**Table files** (`docs/model/tables/*.md`):
- Must contain `## Equation Context`
- Must contain `## Breakpoints`
- Must contain `## References`
- Must have a metadata line containing `**Status:**` with one of: `Exact match`, `Intentional deviation`, `Custom / no reference`

**Sector files** (`docs/model/sectors/*.md`):
- Must contain `## Overview`
- Must contain `## State Variables`
- Must contain `## Governing Equations`
- Must contain `## Lookup Tables`
- Must contain `## References`

**Parameter files** (`docs/model/parameters/*.md`):
- Must contain `## Equation Context`
- Must contain `## Calibration`
- Must contain `## References`

Report files with missing headings.

## Phase 3 — Code-Doc Sync

1. For each lookup table:
   - Parse the `LookupTable::new(...)` call in `tables.rs` to extract x-values and y-values.
   - Parse the `## Breakpoints` markdown table in the corresponding doc file. Extract the "Macroco" column values (or the single value column for exact-match tables).
   - Compare. Flag any mismatch.
2. For each parameter:
   - Parse `ScenarioParams::default()` to extract the default value.
   - Parse the `**BAU value:**` line in the doc file.
   - Compare. Flag any mismatch.
3. Check that `**Source code:**` paths in doc files point to files that exist.

Report all mismatches.

## Phase 4 — Reference Integrity

1. Use WebFetch to get the pyworld3 reference JSON from:
   `https://raw.githubusercontent.com/cvanwynsberghe/pyworld3/master/pyworld3/functions_table_world3.json`
2. For each table file with status "Exact match":
   - Verify breakpoints match pyworld3 values.
   - If they don't match, flag as "claimed exact match but differs."
3. For each table file with status "Intentional deviation":
   - Verify a `## Deviation Rationale` section exists (or equivalent rationale prose under `## Breakpoints`).
4. Read `docs/model/README.md`. Find the deviation summary section (marked `<!-- audit:deviation-summary -->`). Verify counts match actual file statuses.
5. If counts are wrong, update the summary table.

Use this name mapping for pyworld3 comparison:

| Doc file field | pyworld3 key(s) |
|---|---|
| `mortality_0_14` | `m1` |
| `mortality_15_44` | `m2` |
| `mortality_45_64` | `m3` |
| `mortality_65_plus` | `m4` |
| `life_exp_multiplier_food` | `lmf` |
| `life_exp_multiplier_health` | `lmhs2` |
| `crowding_multiplier_ind` | `cmi` |
| `fraction_population_urban` | `fpu` |
| `life_exp_multiplier_pollution` | `lmp` |
| `desired_family_size` | `sfsn` (× dcfsn=3.8) |
| `family_planning_multiplier` | `frsn` |
| `fecundity_multiplier` | `fm` |
| `compensatory_fertility` | `cmple` |
| `health_services_per_capita` | `hsapc` |
| `consumption_fraction` | `fioacv` |
| `industrial_fraction_to_agriculture` | `fioaa1` |
| `indicated_food_per_capita` | `ifpc1` |
| `industrial_fraction_to_services` | `fioas1` |
| `indicated_service_per_capita` | `isopc1` |
| `jobs_per_capital` | `jpicu` |
| `land_yield_multiplier_capital` | `lymc` |
| `land_yield_multiplier_pollution` | `lymap1` |
| `land_fertility_degradation` | `lfdr` |
| `land_fertility_regeneration_time` | `lfrt` |
| `fraction_land_maintenance` | `falm` |
| `capital_fraction_resource_extraction` | `fcaor1` |
| `pollution_assimilation_time` | `ahlm` |
| `urban_industrial_land_per_capita` | `uilpc` |

Tables not in this mapping are "Custom / no reference" and skip pyworld3 comparison.

## Phase 5 — Worktree Diff Awareness

1. Run `git diff --name-only $(git merge-base HEAD origin/main)..HEAD` to find changed files.
2. For each changed `.rs` file:
   - If in `crates/world3-core/src/model/sectors/` → check the corresponding `docs/model/sectors/<name>.md` is also changed.
   - If `lookup/tables.rs` → parse the diff to identify which specific tables changed, check those doc files.
   - If `model/params.rs` → parse the diff to identify which fields changed, check those doc files.
3. Flag any code change without a corresponding doc change.

## Output

After all phases, output a summary:

```
Model Documentation Audit — YYYY-MM-DD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Phase 1 (Completeness):  ✓ 34 tables, 15 parameters, 5 sectors
Phase 2 (Template):      ✓ All files conform
Phase 3 (Code-Doc Sync): ✗ 2 mismatches found
  - tables/desired-family-size.md: y[2] is 3.00 but code has 3.10
  - parameters/technology-growth-rate.md: BAU value 0.014 but code has 0.015
Phase 4 (References):    ✓ pyworld3 comparison up to date
Phase 5 (Diff):          ✓ No undocumented code changes

Overall: FAIL (Phase 3)
```

In `--check` mode: output the summary and exit. Do not offer fixes.

In default mode: after the summary, offer to fix each issue found:
- Breakpoint mismatches: regenerate the `## Breakpoints` table from code
- BAU value mismatches: update the `**BAU value:**` line
- Missing files: create from template with values from code
- Summary count mismatches: regenerate the deviation summary

In `--diff` mode: only run phases 1-3 and 5 on files corresponding to changed code. Skip phase 4 (pyworld3 fetch). Report only relevant issues.
