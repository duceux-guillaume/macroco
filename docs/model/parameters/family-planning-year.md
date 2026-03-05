# Family Planning Year

The year at which family planning programs reach full effectiveness. Before this year, the family planning multiplier on fertility is ramped in linearly from zero at 1900. After this year, the full efficacy is applied.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `family_planning_year`
**BAU value:** 2000.0

## Equation Context

The family planning ramp function controls how quickly programs take effect:

$$
\text{ramp}(t) = \min\!\left(1,\; \frac{t - 1900}{t_{\text{fp}} - 1900}\right)
$$

$$
e_{\text{fp}} = e_{\text{max}} \times \text{ramp}(t)
$$

where $t_{\text{fp}}$ is this parameter and $e_{\text{max}}$ is the [family planning efficacy](family-planning-efficacy.md). The effective planning input $e_{\text{fp}}$ is then passed to the [family planning multiplier](../tables/family-planning-multiplier.md) table to determine fertility reduction. See [Population sector](../sectors/population.md).

## Calibration

In the Collapse scenario, this parameter is set to 2000.0, but since family planning efficacy is 0.0, the year has no effect --- the ramp multiplied by zero is always zero. The parameter only becomes active in scenarios with nonzero efficacy.

In the Ecotopia scenario, `family_planning_year` = 1975.0 and `family_planning_efficacy` = 0.95, meaning full family planning effectiveness is reached by 1975, consistent with the historical timing of major family planning programs in developing nations (1960s--1970s).

The parameter range in the UI is 1950--2100 with a step size of 5 years.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
