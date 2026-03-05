# Family Planning Multiplier (FRSN)

The multiplier on desired fertility from family planning programs. At zero effectiveness (no programs), the multiplier is 1.0 (no reduction). At full effectiveness, fertility is reduced to 40% of the unplanned level. The relationship is concave --- the first increments of family planning are less effective than later ones, reflecting the threshold effects of program reach and acceptance.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `family_planning_multiplier`
**Status:** Intentional deviation (structural)

## Equation Context

This table provides $\text{FRSN}(e_{\text{fp}})$ in the desired fertility equation:

$$
\text{TFR}_{\text{desired}} = \text{DCFS}(\text{DIOPC}) \times \text{CMPLE}(\text{PLE}) \times \text{FRSN}(e_{\text{fp}}) \times \text{FFM}(f_r)
$$

where the effective family planning input is:

$$
e_{\text{fp}} = e_{\text{max}} \times \min\!\left(1,\; \frac{t - 1900}{t_{\text{fp}} - 1900}\right)
$$

$e_{\text{max}}$ is the [family planning efficacy](../parameters/family-planning-efficacy.md) parameter and $t_{\text{fp}}$ is the [family planning year](../parameters/family-planning-year.md). See [Population sector](../sectors/population.md).

## Breakpoints

| Effectiveness | Macroco | | Income expectation | pyworld3 |
|--------------:|--------:|-|-------------------:|---------:|
| 0.00 | 1.00 | | -0.2 | 0.50 |
| 0.25 | 0.90 | | -0.1 | 0.60 |
| 0.50 | 0.75 | | 0.0 | 0.70 |
| 0.75 | 0.55 | | 0.1 | 0.85 |
| 1.00 | 0.40 | | 0.2 | 1.00 |

The two tables have entirely different x-axes and are not directly comparable breakpoint-by-breakpoint.

## Deviation Rationale

World3-03's FRSN table uses the family income expectation difference (negative = income below expectations, positive = income above) as its input. When income falls below expectations, families have fewer children; when income exceeds expectations, they have more. This mechanism operates as a social norm adjustment.

Our implementation uses a simpler 0--1 effectiveness scale controlled by the `family_planning_efficacy` parameter and a linear ramp-in over time. This structural change was made because:

1. The income-expectation mechanism requires additional state (expected income, adaptation rate) not present in our model structure.
2. In Collapse, family planning efficacy is 0.0, so FRSN = 1.0 regardless of table shape --- the table only matters in the Stabilized scenario.
3. The 0--1 scale provides a more intuitive policy lever for the interactive UI.

At Collapse settings (efficacy = 0), both implementations produce FRSN = 1.0, so the deviation has no effect on the standard run.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `frsn`
