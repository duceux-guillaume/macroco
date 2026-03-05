# LFRT --- Land Fertility Regeneration Time

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_fertility_regeneration_time`)
**World3-03 name:** LFRT
**Status:** Exact match with pyworld3

## Equation Context

$$\text{regeneration} = \frac{600 - LFERT}{\mathrm{LFRT}(\text{FALM})}$$

LFRT maps the fraction of agricultural output allocated to land maintenance (FALM) to the time required for land fertility to regenerate toward its inherent level of 600 kg/ha/yr. The table is evaluated in the agriculture sector (`crates/world3-core/src/model/sectors/agriculture.rs`).

## Purpose

Maps the fraction of agricultural output allocated to land maintenance (FALM) to the time required to regenerate land fertility back toward its inherent level of 600 kg/ha/yr. Higher maintenance investment leads to faster regeneration.

When no output is allocated to maintenance (FALM = 0), regeneration takes 20 years. With maximum maintenance effort (FALM = 0.10), regeneration takes only 2 years. The curve has a sharp transition: even small maintenance allocations (FALM = 0.02) cut regeneration time to 13 years.

The regeneration equation drives fertility back toward 600 kg/ha/yr:

$$\text{regeneration} = \frac{600 - LFERT}{\mathrm{LFRT}(\text{FALM})}$$

When $$LFERT = 600$$, regeneration is zero regardless of LFRT.

## Breakpoints

| $$x$$ (FALM fraction) | $$y$$ (regeneration time, years) |
|---|---|
| 0.00 | 20 |
| 0.02 | 13 |
| 0.04 | 8 |
| 0.06 | 4 |
| 0.08 | 2 |
| 0.10 | 2 |

6 points. The floor at 2 years (for FALM $$\geq$$ 0.08) represents a physical limit on how fast soil can recover even with maximum investment.

## References

- Meadows et al. (2004), Table LFRT
- pyworld3: `functions_table_world3.json`, key `LFRT`
