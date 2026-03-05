# Industrial Capital Depreciation Rate

The annual fraction of industrial capital stock that wears out and must be replaced. A higher depreciation rate means factories and machines have shorter useful lifetimes, requiring more investment just to maintain the existing capital stock.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `industrial_depreciation_rate`
**BAU value:** `1.0 / 13.0` (≈ 0.0769 yr⁻¹, corresponding to 13-year lifetime; World3-03 uses 14 years)

## Equation Context

The depreciation rate appears in the industrial capital accumulation equation:

$$\frac{d(IC)}{dt} = IO \cdot f_{inv} - IC \cdot \delta_i$$

A higher $$\delta_i$$ increases the gross investment needed just to maintain the capital stock at its current level. The net effect is to slow capital accumulation and reduce the peak IOPC.

## Deviation Rationale

World3-03 specifies an average industrial capital lifetime of 14 years ($$alic_1 = 14$$), giving a depreciation rate of $$1/14$$. Our model uses 13 years ($$1/13$$), a modest increase of approximately 7.7% in the depreciation rate.

This adjustment compensates for structural differences in our model's population dynamics. Changes to the life expectancy formulation (particularly the Delay3 perceived-LE and EHSPC smooth) produced slightly higher population growth in the early simulation period (1900-1960), which pushed IOPC too high relative to historical data. The faster depreciation dampens early capital accumulation and brings IOPC into better alignment with World Bank historical data.

**Sensitivity:** Low to moderate. The 1-year lifetime reduction shifts IOPC peak by a few percent. The qualitative Collapse trajectory (overshoot and collapse) is unchanged.

## Calibration

Our Collapse value of 13 years (vs. World3-03's 14 years) was calibrated against World Bank IOPC data. The faster depreciation dampens early capital accumulation that otherwise diverges from historical data due to structural differences in population dynamics. The qualitative overshoot-and-collapse trajectory is unchanged.

## Info Panel

**Unit:** fraction/yr

**Beginner:** How fast factories and machines wear out. Higher = capital decays faster, requiring more investment just to maintain.

**Expert:** Used in d(IC)/dt = investment -- IC × depreciation_rate. Default 0.05 = 20-year average capital lifetime.

**Feedback loops:** resource-collapse

**Related variables:** capital.industrial_capital, capital.industrial_output, capital.industrial_output_per_capita

**Impact increase:** Capital wears out faster -- economy needs more investment just to stay level

**Impact decrease:** Capital lasts longer -- more output available for services and consumption

**Sparkline variable:** capital.industrial_output_per_capita

## Slider Range

| Min | Max | Default | Step |
|---|---|---|---|
| 0.02 | 0.15 | 0.0769 | 0.005 |

## References

- Meadows et al. (2004), parameter $$alic_1 = 14$$ years
