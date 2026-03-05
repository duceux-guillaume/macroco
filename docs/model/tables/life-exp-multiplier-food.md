# Life Expectancy Multiplier from Food (LMF)

The food adequacy multiplier on life expectancy. When food per capita is at subsistence level (food ratio = 1), the multiplier is 1.0. Below subsistence, life expectancy drops sharply toward zero. Above subsistence, modest gains reflect the diminishing health returns of food abundance.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `life_exp_multiplier_food`
**Status:** Exact match with World3-03

## Equation Context

This table provides $\text{LMF}(f_r)$ in the life expectancy equation:

$$
\text{LE} = 28 \times \text{LMF}\!\left(\frac{\text{FPC}}{\text{SFPC}}\right) \times \text{LMHS}(\text{EHSPC}) \times \text{LMC} \times \text{LMP}(P_{\text{idx}})
$$

where $f_r = \text{FPC} / \text{SFPC}$ is the food ratio (food per capita divided by subsistence food per capita). See [Population sector](../sectors/population.md).

## Breakpoints

| Food ratio | LMF |
|-----------:|----:|
| 0 | 0.00 |
| 1 | 1.00 |
| 2 | 1.20 |
| 3 | 1.30 |
| 4 | 1.35 |
| 5 | 1.40 |

Matches pyworld3 exactly. Aligned during March 2026 pyworld3 alignment work.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `lmft`
