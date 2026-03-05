# FALM --- Fraction Allocated to Land Maintenance

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`fraction_land_maintenance`)
**World3-03 name:** FALM
**Status:** Exact match with pyworld3

## Equation Context

$$\text{FALM} = \mathrm{FALM\\_table}(\text{food\\_ratio})$$

FALM maps the perceived food ratio ($$FPC_s / SFPC$$) to the fraction of agricultural output allocated to soil maintenance. The FALM value feeds into the [LFRT](land-fertility-regeneration-time.md) table to determine land fertility regeneration speed. Evaluated in the agriculture sector.

## Purpose

Maps the perceived food ratio (smoothed food per capita divided by subsistence food per capita) to the fraction of agricultural output devoted to maintaining soil quality. When food is scarce (low food ratio), societies prioritize immediate food production over long-term soil health. When food is abundant, more resources can be allocated to maintenance.

$$\text{food\\_ratio}_{\text{FALM}} = \frac{FPC_s}{\text{SFPC}}$$

$$\text{FALM} = \mathrm{FALM}(\text{food\\_ratio}_{\text{FALM}})$$

At subsistence (food ratio = 1), only 4% of output goes to maintenance. Even at four times subsistence, the maximum is 10%. This asymmetry captures the tragedy of soil degradation: maintenance is never a priority until it is too late.

The FALM value feeds into the [LFRT](land-fertility-regeneration-time.md) table to determine how quickly land fertility regenerates.

## Breakpoints

| $x$ (food ratio) | $y$ (maintenance fraction) |
|---|---|
| 0 | 0.00 |
| 1 | 0.04 |
| 2 | 0.07 |
| 3 | 0.09 |
| 4 | 0.10 |

5 points. The table saturates at 0.10 for food ratios above 4.

## References

- Meadows et al. (2004), Table FALM
- pyworld3: `functions_table_world3.json`, key `FALM`
