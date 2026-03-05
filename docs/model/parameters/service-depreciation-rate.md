# Service Capital Depreciation Rate

The annual fraction of service capital stock (hospitals, schools, government buildings) that wears out. Service capital has a longer lifetime than industrial capital, reflecting the durability of institutional infrastructure.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `service_depreciation_rate`
**BAU value:** `0.05` (1/20 yr⁻¹, 20-year lifetime; matches World3-03 exactly)

## Equation Context

The depreciation rate appears in the service capital accumulation equation:

$$\frac{d(SC)}{dt} = IO \cdot f_{srv} - SC \cdot \delta_s$$

Service capital is funded by a fraction of industrial output. When the economy contracts, service investment falls while depreciation continues, causing service capital to decline. This reduces service output per capita, which in turn reduces health services and life expectancy — a key channel through which economic collapse raises mortality.

## Calibration

This parameter matches World3-03 exactly ($$alsc_1 = 20$$ years). No calibration adjustment was needed since service capital dynamics align well with historical data at this value.

## Info Panel

**Unit:** fraction/yr

**Beginner:** How fast service infrastructure (hospitals, schools) wears out.

**Expert:** Used in d(SC)/dt = service_investment -- SC × depreciation_rate. Default 0.05.

**Feedback loops:** demographic-transition

**Related variables:** capital.service_output_per_capita, population.life_expectancy

**Impact increase:** Services decay faster -- health and education quality drops

**Impact decrease:** Services last longer -- sustained life expectancy improvements

**Sparkline variable:** capital.service_output_per_capita

## Slider Range

| Min | Max | Default | Step |
|---|---|---|---|
| 0.02 | 0.15 | 0.05 | 0.005 |

## References

- Meadows et al. (2004), parameter $$alsc_1 = 20$$ years
