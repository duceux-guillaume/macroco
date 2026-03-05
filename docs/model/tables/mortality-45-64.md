# Mortality Rate, Ages 45--64 (M3)

Annual mortality rate for the middle-aged cohort (45--64 years) as a function of life expectancy. This cohort has mortality rates comparable to the youngest cohort at low life expectancy, but the gap narrows at higher LE as childhood diseases are controlled while age-related diseases persist.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `mortality_45_64`
**Status:** Exact match with World3-03

## Equation Context

This table provides $M_3(\text{LE})$ in the cohort mortality equation:

$$
D_{45\text{--}64} = C_{45\text{--}64} \times M_3(\text{LE})
$$

See [Population sector](../sectors/population.md).

## Breakpoints

| Life expectancy (years) | M3 (annual rate) |
|------------------------:|-----------------:|
| 20 | 0.0562 |
| 30 | 0.0373 |
| 40 | 0.0252 |
| 50 | 0.0171 |
| 60 | 0.0118 |
| 70 | 0.0083 |
| 80 | 0.0060 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `m3`
