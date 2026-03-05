# LYMC --- Land Yield Multiplier from Capital

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_yield_multiplier_capital`)
**World3-03 name:** LYMC1
**Status:** Exact match with pyworld3

## Purpose

Maps agricultural inputs per hectare (fertilizer, machinery, irrigation spending) to a yield multiplier on land fertility. At zero capital input, land produces at its inherent fertility (multiplier 1.0). The Green Revolution effect appears in the steep initial slope: the first 40 $/ha/yr triples yield. Returns diminish sharply beyond 200 $/ha/yr, reaching a ceiling of 10.0 at 1000 $/ha/yr.

This table is the primary mechanism through which industrial investment increases food production.

## Breakpoints

| $x$ (AIPH, $/ha/yr) | $y$ (multiplier) |
|---|---|
| 0 | 1.0 |
| 40 | 3.0 |
| 80 | 3.8 |
| 120 | 4.4 |
| 160 | 4.9 |
| 200 | 5.4 |
| 240 | 5.7 |
| 280 | 6.0 |
| 320 | 6.3 |
| 360 | 6.6 |
| 400 | 6.9 |
| 440 | 7.2 |
| 480 | 7.4 |
| 520 | 7.6 |
| 560 | 7.8 |
| 600 | 8.0 |
| 640 | 8.2 |
| 680 | 8.4 |
| 720 | 8.6 |
| 760 | 8.8 |
| 800 | 9.0 |
| 840 | 9.2 |
| 880 | 9.4 |
| 920 | 9.6 |
| 960 | 9.8 |
| 1000 | 10.0 |

26 points, uniformly spaced at 40 $/ha/yr intervals.

## Equation Context

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\\_index}) \times \text{agricultural\\_technology}$$

## Audit Notes

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously truncated at $$x = 400$$.

## References

- Meadows et al. (2004), Table LYMC1
- pyworld3: `functions_table_world3.json`, key `LYMC1`
