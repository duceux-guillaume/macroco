# Family Planning Efficacy

The maximum reduction in desired family size achievable through family planning programs. A value of 0.0 means no family planning intervention; a value of 1.0 means maximum possible reduction (FRSN drops to 0.40, reducing fertility by 60%).

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `family_planning_efficacy`
**BAU value:** 0.0

## Equation Context

This parameter scales the input to the [family planning multiplier](../tables/family-planning-multiplier.md) table:

$$
e_{\text{fp}} = e_{\text{max}} \times \min\!\left(1,\; \frac{t - 1900}{t_{\text{fp}} - 1900}\right)
$$

$$
\text{FRSN} = \text{FRSN\_table}(e_{\text{fp}})
$$

where $e_{\text{max}}$ is this parameter and $t_{\text{fp}}$ is the [family planning year](family-planning-year.md). FRSN then multiplies desired fertility:

$$
\text{TFR}_{\text{desired}} = \text{DCFS}(\text{DIOPC}) \times \text{CMPLE}(\text{PLE}) \times \text{FRSN}(e_{\text{fp}}) \times \text{FFM}(f_r)
$$

See [Population sector](../sectors/population.md).

## Calibration

In BAU, efficacy is 0.0. This matches the World3-03 standard run, which assumes no deliberate family planning intervention --- population dynamics are driven entirely by economic development (the demographic transition through DCFS) and compensatory fertility (CMPLE).

In the Stabilized World scenario, efficacy is 0.95, representing aggressive global family planning programs. Combined with `family_planning_year` = 1975.0, this produces FRSN approximately 0.42 by 1975, reducing desired fertility by about 58%.

The parameter is sensitive in scenarios where it is nonzero: changing efficacy from 0.5 to 1.0 can shift equilibrium population by 1--2 billion. It has zero effect in BAU.

The parameter range in the UI is 0.0--1.0 with a step size of 0.05.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
