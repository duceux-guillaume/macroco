# Agricultural Technology

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`agricultural_technology`)
**Unit:** dimensionless multiplier
**Range:** 0.5 -- 3.0
**Step:** 0.1

## Purpose

A direct multiplier on land yield representing crop improvements, irrigation technology, and farming practices beyond what is captured by the capital-input relationship (LYMC). This parameter models the cumulative effect of agricultural research, genetically improved cultivars, and advanced techniques.

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology}$$

At 1.0, yield depends solely on land fertility, capital inputs, and pollution. At 2.0, yield is doubled at every level of capital input --- equivalent to a second Green Revolution.

## Scenario Values

| Scenario | Value | Rationale |
|----------|-------|-----------|
| BAU (Collapse) | 1.0 | No extraordinary agricultural innovation beyond capital-driven gains |
| Technology (Technotopia) | 2.0 | Aggressive crop science, precision agriculture, GMOs |
| Stabilized (Ecotopia) | 2.0 | Sustainable intensification, agroecology at scale |

## Equation Context

The agricultural technology multiplier enters the land yield equation:

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology}$$

At 1.0, yield depends solely on fertility, capital inputs, and pollution. Values above 1.0 uniformly scale yield at all input levels.

## Calibration

BAU uses 1.0, matching the World3-03 standard run (no extraordinary agricultural innovation). Technology and Stabilized presets use 2.0, representing a second Green Revolution. No calibration deviation from World3-03 is needed for BAU.

## Sensitivity

Doubling this parameter doubles food production at all input levels, which delays the food crisis in overshoot scenarios. However, higher yields also increase the erosion multiplier (via the [LERD](../tables/land-erosion-multiplier.md) table), partially offsetting the gain through accelerated soil loss. The net effect is a postponement rather than prevention of agricultural decline unless combined with land protection.

## References

- Meadows et al. (2004), technology scenarios (Chapters 6--7)
