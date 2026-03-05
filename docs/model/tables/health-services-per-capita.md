# Health Services Allocations Per Capita (HSAPC)

Maps service output per capita to health spending per capita. As a society's service sector grows, a portion is allocated to health care (hospitals, sanitation, public health programs). The relationship saturates at approximately 230 USD/person/year --- beyond a certain level of service output, additional spending yields diminishing health infrastructure gains.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `health_services_per_capita`
**Status:** Exact match (aligned) with World3-03

## Equation Context

This table provides the raw health services input that is then smoothed and passed to the life expectancy multiplier:

$$
\text{HSAPC} = \text{HSAPC\_table}(\text{SOPC}) \times h_{\text{mult}}
$$

$$
\frac{d(\text{EHSPC})}{dt} = \frac{\text{HSAPC} - \text{EHSPC}}{20}
$$

where $h_{\text{mult}}$ is the [health investment multiplier](../parameters/health-investment-multiplier.md) parameter and EHSPC (effective health services per capita) is the 20-year smoothed value used by [LMHS](life-exp-multiplier-health.md). See [Population sector](../sectors/population.md).

## Breakpoints

| SOPC (USD/person/yr) | HSAPC (USD/person/yr) |
|---------------------:|----------------------:|
| 0 | 0 |
| 250 | 20 |
| 500 | 50 |
| 750 | 95 |
| 1000 | 140 |
| 1250 | 175 |
| 1500 | 200 |
| 1750 | 220 |
| 2000 | 230 |

Matches pyworld3 exactly. Replaced an earlier custom FSH (fraction of services to health) lookup with the World3-03 HSAPC table, which maps service output per capita directly to health spending per capita.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `hsapc`
