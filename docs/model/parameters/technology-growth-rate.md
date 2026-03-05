# Technology Growth Rate

The annual rate of total factor productivity (TFP) improvement in industrial output, applied as a compound growth multiplier from 1970 onward. This parameter represents the real-world productivity gains from technological progress — better machines, more efficient processes, information technology — that the original 1972 World3 model did not anticipate.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `technology_growth_rate`
**BAU value:** `0.014` (1.4%/yr; World3-03 standard run uses 0)

## Equation Context

The technology growth rate enters the industrial output equation through a compound multiplier:

$$T_m = (1 + g)^{\max(t - 1970,\; 0)}$$

$$IO = \frac{IC \cdot (1 - FCAOR) \cdot T_m}{ICOR}$$

Before 1970, the multiplier is 1.0 (no effect). After 1970, output per unit of productive capital grows at rate $$g$$ annually. By 2000, the multiplier reaches $$(1.014)^{30} \approx 1.52$$; by 2050, $$(1.014)^{80} \approx 3.04$$.

## Deviation Rationale

The original World3 standard run assumes no improvement in industrial productivity. This was a deliberate modeling choice by Meadows et al. — the Collapse scenario asks "what happens with no policy intervention and no lucky technological breakthroughs?"

However, real-world TFP growth has averaged approximately 1.5% per year in developed economies since 1970. Without accounting for this, the model's industrial output per capita diverges sharply from historical data after 1970. The 1.4% rate was calibrated against World Bank IOPC data (1960-2023) to achieve an RMSE below 19% (REQ-026).

The ISOPC dynamic lookup (added in March 2026) captures service-capital feedback that was previously absorbed by the technology rate. This allowed the rate to be reduced from earlier calibration values while maintaining historical fit.

## Calibration

The Collapse rate of 1.4%/yr was calibrated against World Bank IOPC data (1960-2023) to achieve RMSE below 19% (REQ-026). The ISOPC dynamic lookup (March 2026) captures service-capital feedback that was previously absorbed by the technology rate, allowing it to be reduced from earlier calibration values. This parameter is highly sensitive: values above 2% cause a qualitative bifurcation (population peak shifts from ~2030 to ~2073).

## Sensitivity

This parameter has **high sensitivity** and can cause qualitative bifurcations in model behavior:

- At $$g > 0.02$$, the population peak shifts from approximately 2030 to approximately 2073, because sustained industrial growth delays the onset of resource-driven collapse.
- At $$g = 0$$, the model reproduces the original World3 standard run trajectory with an earlier and sharper collapse.
- The transition between "early collapse" and "delayed collapse" regimes is nonlinear — small changes near the bifurcation threshold produce large shifts in trajectory shape.

Always run `cargo test -p world3-cli --test qualitative_dynamics` after changing this parameter to verify that the Collapse overshoot-and-collapse shape is preserved.

## Slider Range

| Min | Max | Default | Step |
|---|---|---|---|
| 0.0 | 0.03 | 0.014 | 0.001 |

## References

- Meadows et al. (1972), standard run assumes zero technology progress
- World Bank total factor productivity estimates (~1.5%/yr for OECD economies, 1970-2020)
