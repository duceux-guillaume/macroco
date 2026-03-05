# Health Investment Multiplier

Scales the health services spending derived from service output per capita. A value of 1.0 represents the baseline allocation. Values above 1.0 represent increased health investment (e.g., policy prioritizing healthcare), while values below 1.0 represent reduced investment.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `health_investment_multiplier`
**BAU value:** 1.0

## Equation Context

This parameter scales the output of the [HSAPC](../tables/health-services-per-capita.md) lookup table before it enters the EHSPC smoothing delay:

$$
\text{HSAPC}_{\text{effective}} = \text{HSAPC\_table}(\text{SOPC}) \times h_{\text{mult}}
$$

$$
\frac{d(\text{EHSPC})}{dt} = \frac{\text{HSAPC}_{\text{effective}} - \text{EHSPC}}{20}
$$

EHSPC then feeds into the [life expectancy multiplier from health services](../tables/life-exp-multiplier-health.md) (LMHS), which multiplies the base life expectancy. See [Population sector](../sectors/population.md).

## Calibration

The Collapse value of 1.0 represents the standard World3-03 health services allocation. No World3-03 scenario modifies this parameter --- it is a Macroco extension that provides an additional policy lever for exploring health investment scenarios.

Sensitivity: increasing the multiplier from 1.0 to 2.0 roughly doubles HSAPC, which shifts EHSPC from approximately 50 to approximately 100 at peak (depending on SOPC trajectory). This moves the LMHS multiplier from approximately 1.8 to approximately 2.0, increasing life expectancy by roughly 3--5 years. The effect is modulated by the 20-year EHSPC smoothing delay.

The parameter range in the UI is 0.5--3.0 with a step size of 0.1.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
