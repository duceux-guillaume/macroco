# Service Capital Depreciation Rate

The annual fraction of service capital stock (hospitals, schools, government buildings) that wears out. Service capital has a longer lifetime than industrial capital, reflecting the durability of institutional infrastructure.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `service_depreciation_rate`

## Value

| | Lifetime | Rate |
|---|---|---|
| **BAU (ours)** | 20 years | $$\delta_s = 1/20 = 0.05 \;\text{yr}^{-1}$$ |
| **World3-03** | 20 years ($$alsc_1 = 20$$) | $$1/20 = 0.05 \;\text{yr}^{-1}$$ |

This parameter matches World3-03 exactly.

## Equation Context

The depreciation rate appears in the service capital accumulation equation:

$$\frac{d(SC)}{dt} = IO \cdot f_{srv} - SC \cdot \delta_s$$

Service capital is funded by a fraction of industrial output. When the economy contracts, service investment falls while depreciation continues, causing service capital to decline. This reduces service output per capita, which in turn reduces health services and life expectancy — a key channel through which economic collapse raises mortality.

## Calibration

This parameter matches World3-03 exactly ($$alsc_1 = 20$$ years). No calibration adjustment was needed since service capital dynamics align well with historical data at this value.

## Slider Range

| Min | Max | Default | Step |
|---|---|---|---|
| 0.02 | 0.15 | 0.05 | 0.005 |

## References

- Meadows et al. (2004), parameter $$alsc_1 = 20$$ years
