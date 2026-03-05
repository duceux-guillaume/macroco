# Agricultural Technology

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`agricultural_technology`)
**BAU value:** `1.0` (no extraordinary agricultural innovation; matches World3-03 standard run)

## Equation Context

The agricultural technology multiplier enters the land yield equation:

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology}$$

At 1.0, yield depends solely on fertility, capital inputs, and pollution. Values above 1.0 uniformly scale yield at all input levels.

## Calibration

Collapse uses 1.0, matching the World3-03 standard run (no extraordinary agricultural innovation). Technology and Stabilized presets use 2.0, representing a second Green Revolution. No calibration deviation from World3-03 is needed for Collapse.

## Sensitivity

Doubling this parameter doubles food production at all input levels, which delays the food crisis in overshoot scenarios. However, higher yields also increase the erosion multiplier (via the [LERD](../tables/land-erosion-multiplier.md) table), partially offsetting the gain through accelerated soil loss. The net effect is a postponement rather than prevention of agricultural decline unless combined with land protection.

## References

- Meadows et al. (2004), technology scenarios (Chapters 6--7)
