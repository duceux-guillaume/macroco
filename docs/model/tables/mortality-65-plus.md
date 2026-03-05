# Mortality Rate, Ages 65+ (M4)

Annual mortality rate for the senior cohort (65+ years) as a function of life expectancy. This cohort has the highest mortality rates, ranging from 13% per year at LE = 20 to 4% per year at LE = 80. The 65+ cohort has no aging-out flow --- members remain until death.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `mortality_65_plus`
**Status:** Exact match with World3-03

## Equation Context

This table provides $M_4(\text{LE})$ in the cohort mortality equation:

$$
D_{65+} = C_{65+} \times M_4(\text{LE})
$$

See [Population sector](../sectors/population.md).

## Breakpoints

| Life expectancy (years) | M4 (annual rate) |
|------------------------:|-----------------:|
| 20 | 0.13 |
| 30 | 0.11 |
| 40 | 0.09 |
| 50 | 0.07 |
| 60 | 0.06 |
| 70 | 0.05 |
| 80 | 0.04 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `m4`
