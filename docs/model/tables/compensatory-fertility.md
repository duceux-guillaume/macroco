# Compensatory Multiplier from Perceived Life Expectancy (CMPLE)

The compensatory fertility multiplier. When perceived life expectancy is low (indicating high infant and child mortality), women have more children to compensate for expected deaths --- a well-documented demographic phenomenon. At very low perceived LE (0 years), the multiplier is 3.0 (tripling desired family size). As perceived LE rises toward 80, the multiplier approaches 1.0 (no compensation needed).

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `compensatory_fertility`
**Status:** Exact match with World3-03

## Equation Context

This table provides $\text{CMPLE}(\text{PLE})$ in the desired fertility equation:

$$
\text{TFR}_{\text{desired}} = \text{DCFS}(\text{DIOPC}) \times \text{CMPLE}(\text{PLE}) \times \text{FRSN}(e_{\text{fp}}) \times \text{FFM}(f_r)
$$

The input is perceived life expectancy (PLE), which is a 20-year Delay3 of actual life expectancy. The delay is critical: even after actual LE improves, perceived LE lags behind, keeping CMPLE elevated and sustaining higher fertility for approximately two decades. See [Population sector](../sectors/population.md).

## Breakpoints

| Perceived LE (years) | CMPLE |
|---------------------:|------:|
| 0 | 3.00 |
| 10 | 2.10 |
| 20 | 1.60 |
| 30 | 1.40 |
| 40 | 1.30 |
| 50 | 1.20 |
| 60 | 1.10 |
| 70 | 1.05 |
| 80 | 1.00 |

Matches pyworld3 exactly. Aligned during March 2026 pyworld3 alignment work. Previously had a reduced range (1.40--0.90) and fewer breakpoints.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `cmple`
