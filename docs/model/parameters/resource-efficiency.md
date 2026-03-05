# Resource Efficiency

The resource efficiency parameter represents all forms of extraction and utilization efficiency: better mining technology, material substitution, recycling, lighter-weight manufacturing, and more efficient energy conversion. Doubling this parameter exactly halves the resource depletion rate.

**Sector:** [Resources](../sectors/resources.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `resource_efficiency`
**BAU value:** `1.05` (5% uplift over World3-03 default of 1.0)

## Equation Context

The resource efficiency parameter $$r_e$$ divides the extraction rate in the resources sector ODE:

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$

A value of 2.0 halves the depletion rate at any given population and IOPC level. This is verified by the unit test `test_resource_derivative_efficiency_halves_rate` in the resources sector.

## Calibration

Collapse uses $$r_e = 1.05$$ (a 5% uplift over the World3-03 default of 1.0). The original World3-03 standard run uses $$r_e = 1.0$$, reflecting the 1972 assumption that extraction efficiency would not improve. Our deviation compensates for:

1. **Historical calibration.** Real-world TFP growth (~1.5%/yr in mining and energy sectors since 1970). The 5% uplift brings NNR depletion into alignment with observed data (RMSE = 0.9%).

2. **IOPC trajectory.** Without the adjustment, NNR depletion constrains industrial output too early, causing IOPC to diverge from World Bank data in 1990–2010.

The deviation is intentionally small. At $$r_e = 1.05$$, the qualitative Collapse overshoot-and-collapse trajectory is preserved — resources still deplete to roughly 11% by 2100. Larger values (e.g., Technology preset's 4.0) fundamentally alter the trajectory.

## Sensitivity

Resource efficiency interacts with the FCAOR table to determine when capital diversion becomes significant. Under Collapse:

- At $$r_e = 1.0$$: NNR falls below 50% around 2040, triggering rapid capital diversion.
- At $$r_e = 1.05$$: The 50% threshold is reached a few years later, modestly delaying the onset of collapse.
- At $$r_e = 4.0$$: NNR remains above 70% through 2100; resource constraints never dominate. Collapse, if it occurs, is driven by pollution or food limits instead.

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*. Technology scenario parameters.
- Historical calibration: `cargo test -p world3-core --test historical_calibration` (NNR RMSE = 0.9%).
