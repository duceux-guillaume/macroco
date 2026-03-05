# Agricultural Technology Growth Rate

Annual rate of improvement in agricultural yield from Green Revolution advances and modern farming techniques. This is a Macroco extension beyond World3-03, which has no equivalent parameter — the original model treats agricultural technology as a static multiplier.

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `agricultural_technology_growth_rate`
**BAU value:** `0.005` (0.5%/yr compound growth from 1960; Macroco extension, no World3-03 equivalent)

## Equation Context

The growth rate compounds from 1960 to produce a time-varying multiplier on land yield:

$$T_{ag} = (1 + g_{ag})^{\max(t - 1960,\; 0)}$$

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology} \times T_{ag}$$

Before 1960, the multiplier is 1.0. The Collapse preset uses $$g_{ag} = 0.005$$, giving a multiplier of ~1.37 by 2020 and ~2.03 by 2100. Technology and Stabilized presets use 0.0 (relying on the static `agricultural_technology = 2.0` multiplier instead).

## Calibration

The Collapse value of 0.5%/yr was calibrated against FAO food production data (1960–2023). USDA Economic Research Service estimates agricultural TFP growth at roughly 1%/yr globally, but LYMC already captures input-driven gains (capital investment in agriculture), so the growth rate parameter covers only the residual TFP from breeding, agronomy, and technique improvements. The 0.5% rate brings food per capita into alignment with historical data (RMSE < 21%).

Technology and Stabilized presets set this to 0.0 because they use a static `agricultural_technology = 2.0` multiplier that already represents a step-change in agricultural capability.

## References

- USDA Economic Research Service: International Agricultural Productivity (TFP ~1%/yr global average)
- FAO FAOSTAT food production indices (1961–2023)
- Meadows et al. (2004) — no equivalent parameter; agricultural technology is static in World3-03
