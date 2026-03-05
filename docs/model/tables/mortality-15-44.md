# Mortality Rate, Ages 15--44 (M2)

Annual mortality rate for the young adult cohort (15--44 years) as a function of life expectancy. This cohort has the lowest mortality rates of all four age groups, reflecting the biological robustness of young adulthood. At LE = 20, the rate is 2.66% per year; at LE = 80, it falls to 0.08% per year.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `mortality_15_44`
**Status:** Exact match with World3-03

## Equation Context

This table provides $M_2(\text{LE})$ in the cohort mortality equation:

$$
D_{15\text{--}44} = C_{15\text{--}44} \times M_2(\text{LE})
$$

See [Population sector](../sectors/population.md).

## Breakpoints

| Life expectancy (years) | M2 (annual rate) |
|------------------------:|-----------------:|
| 20 | 0.0266 |
| 30 | 0.0171 |
| 40 | 0.0110 |
| 50 | 0.0065 |
| 60 | 0.0040 |
| 70 | 0.0016 |
| 80 | 0.0008 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `m2`
