# UILPC --- Urban-Industrial Land Per Capita

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`urban_industrial_land_per_capita`)
**World3-03 name:** UILPC
**Status:** Exact match with pyworld3

## Equation Context

$$UIL = \mathrm{UILPC}(\mathrm{IOPC}) \times \text{population}$$

UILPC maps industrial output per capita to the hectares of land per person required for urban and industrial use. The total urban-industrial land demand competes with arable land. Evaluated in the agriculture sector (`crates/world3-core/src/model/sectors/agriculture.rs`).

## Purpose

Maps industrial output per capita (IOPC) to the hectares of land per person required for urban and industrial use (cities, factories, roads, infrastructure). As societies industrialize, per-capita land demand rises from 0.005 ha/person at subsistence to 0.09 ha/person at high income.

Urban-industrial land is taken from the arable land stock, creating a direct competition between food production and urbanization. In the Collapse scenario, UIL expansion is modest in the early period but accelerates with industrialization.

$$UIL_{\text{desired}} = \mathrm{UILPC}(\mathrm{IOPC}) \times \text{population}$$

## Breakpoints

| $$x$$ (IOPC, $/person/yr) | $$y$$ (ha/person) |
|---|---|
| 0 | 0.005 |
| 200 | 0.008 |
| 400 | 0.015 |
| 600 | 0.025 |
| 800 | 0.040 |
| 1000 | 0.055 |
| 1200 | 0.070 |
| 1400 | 0.080 |
| 1600 | 0.090 |

9 points, uniformly spaced at 200 $/person/yr intervals.

## Audit Notes

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had dramatically higher values (2--6 times pyworld3).

## References

- Meadows et al. (2004), Table UILPC
- pyworld3: `functions_table_world3.json`, key `UILPC`
