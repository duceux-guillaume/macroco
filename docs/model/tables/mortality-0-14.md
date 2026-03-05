# Mortality Rate, Ages 0--14 (M1)

Annual mortality rate for the youngest age cohort (0--14 years) as a function of life expectancy. At low life expectancy (LE = 20), child mortality is extremely high at 5.67% per year. As life expectancy improves through better food, health services, and lower pollution, child mortality falls rapidly --- reaching 0.1% per year at LE = 80.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `mortality_0_14`
**Status:** Exact match with World3-03

## Equation Context

This table provides $M_1(\text{LE})$ in the cohort mortality equation:

$$
D_{0\text{--}14} = C_{0\text{--}14} \times M_1(\text{LE})
$$

See [Population sector](../sectors/population.md).

## Breakpoints

| Life expectancy (years) | M1 (annual rate) |
|------------------------:|-----------------:|
| 20 | 0.0567 |
| 30 | 0.0366 |
| 40 | 0.0243 |
| 50 | 0.0155 |
| 60 | 0.0082 |
| 70 | 0.0023 |
| 80 | 0.0010 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `m1`
