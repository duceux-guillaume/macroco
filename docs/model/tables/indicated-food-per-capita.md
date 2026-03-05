# Indicated Food Per Capita (IFPC)

The level of food per capita that a society demands given its industrial development. As IOPC rises, populations expect better diets — more protein, more variety, more processing — so indicated food consumption increases with income.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `indicated_food_per_capita`
**Status:** Exact match

## Equation Context

IFPC enters the agricultural allocation decision as the denominator of the food ratio:

$$\text{food\\_ratio} = \frac{FPC_{smooth}}{IFPC(IOPC)}$$

$$f_{agr} = \text{FIOAA}(\text{food\\_ratio})$$

At subsistence levels ($$IOPC = 0$$), indicated food is 230 kg/person/yr — the minimum for basic health. At high industrialization ($$IOPC = 1600$$), indicated food rises to 1250 kg/person/yr, reflecting the dietary standards of wealthy nations. This scaling keeps the food ratio moderate even when food production is high, preventing the [fraction to agriculture](industrial-fraction-to-agriculture.md) from collapsing to zero in high-IOPC scenarios.

## Breakpoints

| IOPC ($/person/yr) | IFPC (kg/person/yr) |
|---|---|
| 0 | 230 |
| 200 | 480 |
| 400 | 690 |
| 600 | 850 |
| 800 | 970 |
| 1000 | 1070 |
| 1200 | 1150 |
| 1400 | 1210 |
| 1600 | 1250 |

This table is an exact match to pyworld3's IFPC1 table, aligned during the March 2026 pyworld3 alignment work.

## References

- Meadows et al. (2004), Table IFPC1 (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `ifpc1`
