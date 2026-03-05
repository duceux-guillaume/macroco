# Land Protection Fraction

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`land_protection_fraction`)
**Unit:** dimensionless fraction
**Range:** 0.0 -- 0.5
**Step:** 0.05

## Purpose

The fraction of arable land under active protection from degradation and overuse. Protected land erodes at a reduced rate, preserving soil quality for future production. This parameter represents policy interventions such as conservation tillage mandates, soil erosion controls, wetland buffers, and protected agricultural zones.

The protection fraction enters the erosion equation as a multiplicative reduction:

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\_ratio}) \times (1 - \text{land\_protection\_fraction})$$

At 0.0, all land erodes at the full rate. At 0.3, erosion is reduced by 30%. The maximum of 0.5 (50% reduction) reflects the practical limit of protection --- some erosion is unavoidable under any farming system.

## Scenario Values

| Scenario | Value | Rationale |
|----------|-------|-----------|
| BAU (Collapse) | 0.0 | No systematic soil conservation policy |
| Technology (Technotopia) | 0.0 | Technology scenario focuses on efficiency, not conservation |
| Stabilized (Ecotopia) | 0.3 | Active soil conservation as part of sustainability policy |

## Equation Context

The land protection fraction enters the erosion equation as a multiplicative reduction:

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\_ratio}) \times (1 - \text{land\_protection\_fraction})$$

At 0.0, all land erodes at the full rate determined by the [LERD](../tables/land-erosion-multiplier.md) table. At the maximum of 0.5, erosion is halved.

## Calibration

BAU uses 0.0 (no protection), matching the World3-03 standard run assumption. The Stabilized preset uses 0.3, representing active soil conservation policy. No deviation from World3-03 is needed for BAU calibration since the parameter is zero in the standard run.

## Sensitivity

At BAU erosion rates, increasing protection from 0.0 to 0.3 extends the productive life of arable land by roughly 40%. This has compounding effects: preserved land maintains food production, reducing pressure to develop marginal land (which is more expensive and less productive). In the Stabilized scenario, land protection is essential for preventing the erosion-fertility reinforcing loop from degrading agricultural capacity.

## References

- Meadows et al. (2004), stabilized world scenario (Chapter 7)
