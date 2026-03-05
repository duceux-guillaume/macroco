# Fraction of Population Urban (FPU)

The fraction of the total population living in urban areas, as a function of total population. Urbanization increases with population size, reflecting historical patterns of rural-to-urban migration as societies grow. The fraction saturates at 0.8, consistent with observed urbanization ceilings in developed nations.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `fraction_population_urban`
**Status:** Exact match with World3-03

## Equation Context

FPU is combined with [CMI](crowding-multiplier-ind.md) (crowding multiplier from industrialization) to compute the life conditions multiplier:

$$
\text{LMC} = \max\!\bigl(0,\; 1 - \text{CMI}(\text{IOPC}) \times \text{FPU}(\text{POP})\bigr)
$$

See [Population sector](../sectors/population.md).

## Breakpoints

| Population (persons) | FPU |
|---------------------:|----:|
| 0 | 0.00 |
| 2 x 10^9 | 0.20 |
| 4 x 10^9 | 0.40 |
| 6 x 10^9 | 0.50 |
| 8 x 10^9 | 0.58 |
| 10 x 10^9 | 0.65 |
| 12 x 10^9 | 0.72 |
| 14 x 10^9 | 0.78 |
| 16 x 10^9 | 0.80 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `fpu`
