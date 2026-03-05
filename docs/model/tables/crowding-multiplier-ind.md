# Crowding Multiplier from Industrialization (CMI)

The crowding mortality index as a function of industrial output per capita. At very low IOPC, crowding is severe (high density, poor sanitation) and the multiplier is strongly positive. At moderate IOPC (400--800), the multiplier goes negative, representing the mortality-reducing effects of urban infrastructure, sanitation, and public health investment. At high IOPC, the multiplier turns positive again as overconsumption and lifestyle diseases emerge.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `crowding_multiplier_ind`
**Status:** Exact match with World3-03

## Equation Context

CMI is combined with [FPU](fraction-population-urban.md) (fraction of population urban) to compute the life conditions multiplier:

$$
\text{LMC} = \max\!\bigl(0,\; 1 - \text{CMI}(\text{IOPC}) \times \text{FPU}(\text{POP})\bigr)
$$

which enters the life expectancy equation. When CMI is negative (mid-IOPC range), LMC exceeds 1.0, boosting life expectancy. See [Population sector](../sectors/population.md).

## Breakpoints

| IOPC (USD/person/yr) | CMI |
|----------------------:|----:|
| 0 | 0.50 |
| 200 | 0.05 |
| 400 | -0.10 |
| 600 | -0.08 |
| 800 | -0.02 |
| 1000 | 0.05 |
| 1200 | 0.10 |
| 1400 | 0.15 |
| 1600 | 0.20 |

Matches pyworld3 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `cmi`
