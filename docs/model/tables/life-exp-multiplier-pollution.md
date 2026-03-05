# Life Expectancy Multiplier from Pollution (LMP)

The pollution multiplier on life expectancy. At low pollution levels, the effect is negligible (multiplier near 1.0). As persistent pollution rises, health impacts become severe --- at a pollution index of 100, life expectancy is reduced to 20% of its unpolluted value. The relationship is nonlinear, with an accelerating decline above a pollution index of about 40.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `life_exp_multiplier_pollution`
**Status:** Exact match with World3-03

## Equation Context

This table provides $\text{LMP}(P_{\text{idx}})$ in the life expectancy equation:

$$
\text{LE} = 28 \times \text{LMF}(f_r) \times \text{LMHS}(\text{EHSPC}) \times \text{LMC} \times \text{LMP}(P_{\text{idx}})
$$

where $P_{\text{idx}}$ is the persistent pollution index (normalized to 1.0 in 1970). See [Population sector](../sectors/population.md).

## Breakpoints

| Pollution index | LMP |
|----------------:|----:|
| 0 | 1.00 |
| 10 | 0.99 |
| 20 | 0.97 |
| 30 | 0.95 |
| 40 | 0.90 |
| 50 | 0.85 |
| 60 | 0.75 |
| 70 | 0.65 |
| 80 | 0.55 |
| 90 | 0.40 |
| 100 | 0.20 |

Matches pyworld3 exactly. Aligned during March 2026 pyworld3 alignment work. Previously had fewer breakpoints and less severe high-pollution effects.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `lmp`
