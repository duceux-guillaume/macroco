# LYMAP --- Land Yield Multiplier from Pollution

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_yield_multiplier_pollution`)
**World3-03 name:** LYMAP1
**Status:** Exact match with pyworld3

## Purpose

Maps the persistent pollution index to a yield multiplier. Pollution degrades crop yields through acid rain, soil contamination, and ozone damage. At low pollution levels (index < 10), yield is unaffected. Beyond index 10, degradation accelerates: yields fall to 70% at index 20 and 40% at index 30.

In the Collapse scenario, pollution index remains below 10 until the mid-21st century, so this table has little effect during the historical period. Its impact becomes severe during the overshoot phase when pollution peaks.

## Breakpoints

| $$x$$ (pollution index) | $$y$$ (multiplier) |
|---|---|
| 0 | 1.0 |
| 10 | 1.0 |
| 20 | 0.7 |
| 30 | 0.4 |

4 points. Beyond $$x = 30$$, the table clamps at $$y = 0.4$$.

## Equation Context

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology}$$

## Audit Notes

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had an extended x-range with more gradual degradation.

## References

- Meadows et al. (2004), Table LYMAP1
- pyworld3: `functions_table_world3.json`, key `LYMAP1`
