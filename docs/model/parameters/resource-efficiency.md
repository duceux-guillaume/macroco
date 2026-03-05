# Resource Efficiency (`resource_efficiency`)

**Source code:** `crates/world3-core/src/model/params.rs`

**Sector:** Resources

**Units:** dimensionless multiplier

**Range:** 1.0 -- 5.0 (UI slider step: 0.25)

## Values by Preset

| Preset | Value | Rationale |
|---|---|---|
| BAU (Collapse) | 1.05 | Slight improvement over World3-03 baseline |
| Technology (Technotopia) | 4.0 | Aggressive efficiency gains |
| Stabilized (Ecotopia) | 4.0 | Aggressive efficiency gains |
| World3-03 original | 1.0 | No efficiency improvement |

## Role in the Model

The resource efficiency parameter $$r_e$$ divides the extraction rate in the resources sector ODE:

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$

A value of $$r_e = 2.0$$ means each unit of industrial output consumes half the resources it would at baseline. The parameter represents all forms of extraction and utilization efficiency: better mining technology, material substitution, recycling, lighter-weight manufacturing, and more efficient energy conversion.

Doubling $$r_e$$ exactly halves the depletion rate at any given level of population and IOPC. This is verified by the unit test `test_resource_derivative_efficiency_halves_rate` in the resources sector.

## Equation Context

The resource efficiency parameter $$r_e$$ divides the extraction rate in the resources sector ODE:

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$

A value of 2.0 halves the depletion rate at any given population and IOPC level. The parameter represents all forms of extraction and utilization efficiency improvements.

## Calibration

BAU uses $$r_e = 1.05$$ (a 5% uplift over the World3-03 default of 1.0). This compensates for real-world TFP growth (~1.5%/yr in mining and energy sectors since 1970) and brings the NNR depletion trajectory into alignment with historical data (NNR fraction RMSE = 0.9%). The qualitative overshoot-and-collapse trajectory is preserved.

## Deviation from World3-03

The original World3-03 BAU scenario uses $$r_e = 1.0$$, reflecting the 1972 assumption that resource extraction efficiency would not improve under business-as-usual conditions. Our BAU preset uses $$r_e = 1.05$$ for two reasons:

1. **Historical calibration.** Real-world resource extraction efficiency has improved modestly since 1970 due to technological progress (total factor productivity growth of roughly 1.5%/yr in mining and energy sectors). The 5% uplift brings the simulated NNR depletion trajectory into closer alignment with observed data (NNR fraction RMSE = 0.9% against historical estimates).

2. **IOPC trajectory.** Without the efficiency adjustment, NNR depletion begins constraining industrial output slightly too early, causing IOPC to diverge from World Bank historical data in the 1990--2010 period.

The deviation is intentionally small. At $$r_e = 1.05$$, the qualitative BAU overshoot-and-collapse trajectory is preserved — resources still deplete to roughly 11% by 2100. Larger values (e.g., the Technology preset's 4.0) fundamentally alter the trajectory by delaying depletion long enough for other constraints (pollution, food) to dominate.

## Sensitivity

Resource efficiency interacts with the FCAOR table to determine when capital diversion becomes significant. Under BAU:

- At $$r_e = 1.0$$: NNR falls below 50% around 2040, triggering rapid capital diversion.
- At $$r_e = 1.05$$: The 50% threshold is reached a few years later, modestly delaying the onset of collapse.
- At $$r_e = 4.0$$: NNR remains above 70% through 2100; resource constraints never dominate. Collapse, if it occurs, is driven by pollution or food limits instead.

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*. Technology scenario parameters.
- Historical calibration: `cargo test -p world3-core --test historical_calibration` (NNR RMSE = 0.9%).
