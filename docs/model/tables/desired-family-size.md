# Desired Completed Family Size (DCFS)

The number of children a woman desires over her lifetime, as a function of perceived industrial output per capita (DIOPC). This table captures the demographic transition: at low income, desired family size is moderate; it peaks at mid-income levels (the "population boom" of early industrialization), then declines as rising income, education, and urbanization shift preferences toward smaller families.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `desired_family_size`
**Status:** Intentional deviation from World3-03

## Equation Context

This table provides $\text{DCFS}(\text{DIOPC})$ in the desired fertility equation:

$$
\text{TFR}_{\text{desired}} = \text{DCFS}(\text{DIOPC}) \times \text{CMPLE}(\text{PLE}) \times \text{FRSN}(e_{\text{fp}}) \times \text{FFM}(f_r)
$$

where DIOPC is a 20-year smooth of IOPC (delayed industrial output per capita), capturing the social adjustment lag in fertility expectations. See [Population sector](../sectors/population.md).

## Breakpoints

| DIOPC (USD/person/yr) | Macroco | World3-03 effective | Delta |
|----------------------:|--------:|----:|------:|
| 0 | 2.85 | 4.75 | -40% |
| 200 | 3.50 | 3.80 | -8% |
| 400 | 3.00 | 3.42 | -12% |
| 600 | 2.42 | 3.04 | -20% |
| 800 | 1.90 | 2.85 | -33% |

World3-03 effective DCFS is computed as $\text{dcfsn} \times \text{SFSN}(\text{DIOPC})$, where dcfsn = 3.8 and SFSN is the social family size norm lookup. Our implementation merges these into a single table. CMPLE (compensatory multiplier from perceived life expectancy) is applied separately on top, matching pyworld3 structure.

## Deviation Rationale

The DCFS table is the primary calibration lever for population dynamics. Our values are lower than World3-03 across the entire DIOPC range, for two reasons:

1. **Delay3 perceived-LE interaction.** Our implementation uses a three-stage Delay3 for perceived life expectancy (matching World3-03 specification), whereas pyworld3 uses a single first-order delay (Delay1). The Delay3 pipeline causes the compensatory fertility multiplier (CMPLE) to remain elevated longer during periods of improving life expectancy, because perceived LE lags actual LE with more inertia. With the original DCFS values, this produced a population spike to 14.15 billion with 79.5% RMSE against historical data.

2. **Non-monotonic shape.** The table peaks at DIOPC = 200 (3.50 children/woman), capturing the mid-income population boom observed in developing nations where mortality has fallen but fertility expectations have not yet adjusted. This hump shape is not present in World3-03's monotonically declining SFSN, but improves historical calibration by concentrating population growth in the 1960--1990 period.

Current calibration results: Population RMSE = 13.2%, peak approximately 8.2 billion at approximately 2082.

**Impact:** High --- this is the primary fertility control table. Changes of even 0.1 in DCFS values can shift population peak by decades or billions.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference keys: `dcfsn`, `sfsn`
