# Life Expectancy Multiplier from Health Services (LMHS2)

The health services multiplier on life expectancy. Reflects the impact of modern medical technology, sanitation, and public health infrastructure on longevity. At zero health spending, the multiplier is 1.0 (subsistence baseline). At high spending (EHSPC = 100), life expectancy doubles.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `life_exp_multiplier_health`
**Status:** Exact match with World3-03 (LMHS2)

## Equation Context

This table provides $\text{LMHS}(\text{EHSPC})$ in the life expectancy equation:

$$
\text{LE} = 28 \times \text{LMF}(f_r) \times \text{LMHS}(\text{EHSPC}) \times \text{LMC} \times \text{LMP}(P_{\text{idx}})
$$

The input is EHSPC (effective health services per capita), a 20-year first-order exponential smooth of HSAPC. See [Population sector](../sectors/population.md).

World3-03 defines two tables: LMHS1 (pre-1940, ceiling 1.8) and LMHS2 (post-1940, ceiling 2.0). We use LMHS2 exclusively, which reflects the post-1940 medical technology regime. The policy switch year (iphst = 1940) is before the model's historical calibration range (1960--2023), so LMHS1 is never active in practice.

## Breakpoints

| EHSPC (USD/person/yr) | LMHS |
|-----------------------:|-----:|
| 0 | 1.00 |
| 20 | 1.40 |
| 40 | 1.60 |
| 60 | 1.80 |
| 80 | 1.95 |
| 100 | 2.00 |

Matches pyworld3 LMHS2 exactly.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `lmhs2`
