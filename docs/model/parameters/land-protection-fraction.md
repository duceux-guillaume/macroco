# Land Protection Fraction

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`land_protection_fraction`)
**BAU value:** `0.0` (no soil conservation policy; matches World3-03 standard run)

## Equation Context

The land protection fraction enters the erosion equation as a multiplicative reduction:

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\_ratio}) \times (1 - \text{land\_protection\_fraction})$$

At 0.0, all land erodes at the full rate determined by the [LERD](../tables/land-erosion-multiplier.md) table. At the maximum of 0.5, erosion is halved.

## Calibration

Collapse uses 0.0 (no protection), matching the World3-03 standard run assumption. The Ecotopia preset uses 0.3, representing active soil conservation policy. No deviation from World3-03 is needed for Collapse calibration since the parameter is zero in the standard run.

## Sensitivity

At Collapse erosion rates, increasing protection from 0.0 to 0.3 extends the productive life of arable land by roughly 40%. This has compounding effects: preserved land maintains food production, reducing pressure to develop marginal land (which is more expensive and less productive). In the Ecotopia scenario, land protection is essential for preventing the erosion-fertility reinforcing loop from degrading agricultural capacity.

## References

- Meadows et al. (2004), stabilized world scenario (Chapter 7)
