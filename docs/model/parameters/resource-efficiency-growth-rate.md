# Resource Efficiency Growth Rate

Annual improvement rate in resource extraction technology, applied from 1970 onward. This is a Macroco extension not present in World3-03.

**Sector:** [Resources](../sectors/resources.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `resource_efficiency_growth_rate`
**BAU value:** `0.0035` (0.35%/yr; Technotopia/Ecotopia use 0.0)

## Equation Context

The growth rate compounds on the base `resource_efficiency` parameter from 1970:

$$r_{e,\text{eff}} = r_e \times (1 + r_{e,\text{growth}})^{\max(\text{year} - 1970, 0)}$$

This effective efficiency divides the extraction rate in the resources sector ODE:

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_{e,\text{eff}}}$$

Before 1970, the growth rate has no effect (the exponent is clamped to zero). This mirrors the `agricultural_technology_growth_rate` pattern (which uses 1960 as its start year).

## Calibration

Real-world extraction technology has improved steadily since the 1970s:

| Domain | Metric | Improvement | Source |
|--------|--------|-------------|--------|
| Oil | Recovery factor (US) | 22% (1979) to 39% (2020), ~1.2%/yr | Oil & Gas Journal |
| Copper | Energy intensity | 196 to 84 MJ/t ore (1930s to 1970s), ~2.1%/yr | MDPI Resources |
| Material | GDP per tonne extracted | ~0.9%/yr improvement (1945-2002) | UNEP Decoupling |

Declining ore grades offset roughly half these gains. Our 0.35%/yr is conservative, below the net real-world rate (~0.5-0.6%/yr), because World3's FCAOR already captures some extraction cost increase.

### Effective Efficiency Over Time (Collapse)

| Year | Effective efficiency |
|------|---------------------|
| 1900-1970 | 1.05 (base, no growth) |
| 2000 | 1.17 |
| 2020 | 1.25 |
| 2050 | 1.39 |
| 2100 | 1.65 |

### Calibration Impact

Adding the growth rate improved IOPC RMSE from 16.5% to 14.9% and NNR RMSE from 1.3% to 1.1%. The 2023 IOPC error improved from -28.9% to -20.2%. All other variables were unaffected.

## Sensitivity

- At 0.0%/yr: baseline (no improvement). IOPC collapses earlier, NNR depletes faster.
- At 0.35%/yr (Collapse default): modest delay to resource-driven collapse. Qualitative trajectory preserved.
- At 0.4%/yr: NNR max-year error exceeds 7% threshold. Too aggressive.
- The parameter interacts with `resource_efficiency` multiplicatively. Technotopia/Ecotopia presets use 0.0 because their static `resource_efficiency=4.0` already represents the policy intervention.

## Info Panel

**Unit:** per year

**Beginner:** Annual improvement in extraction technology from 1970. Real-world mining and drilling have gotten steadily more efficient. Collapse uses 0.35%/yr.

**Expert:** Macroco extension: effective_efficiency = resource_efficiency × (1 + growth_rate)^max(year-1970, 0). Conservative vs net real-world rate (~0.5-0.6%/yr) because FCAOR captures some extraction cost.

**Feedback loops:** resource-collapse, population-resource

**Related variables:** resources.nonrenewable_resources, resources.fraction_remaining, capital.industrial_output_per_capita

**Impact increase:** Resources depleted more slowly -- delays industrial collapse

**Impact decrease:** No extraction improvement -- earlier resource scarcity

**Sparkline variable:** resources.fraction_remaining

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*. Technology scenario parameters.
- Historical calibration: `cargo test -p world3-core --test historical_calibration` (IOPC RMSE = 14.9%, NNR RMSE = 1.1%).
- Unit test: `test_resource_efficiency_growth_rate` in `resources.rs`.
