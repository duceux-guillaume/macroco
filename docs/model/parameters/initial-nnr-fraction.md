# Initial NNR Fraction

The initial fraction of non-renewable resources available at the start of the simulation (year 1900). A value of 1.0 represents the full initial endowment — the World3 assumption that 100% of extractable resources remain at the start of the industrial era.

**Sector:** [Resources](../sectors/resources.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `initial_nnr_fraction`
**BAU value:** `1.0` (matches World3-03 exactly)

## Equation Context

The initial NNR fraction sets the starting value of the non-renewable resources ODE stock at year 1900:

$$NNR(1900) = \texttt{initial\_nnr\_fraction}$$

The stock then evolves according to the depletion equation: $$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$. The fraction remaining ($$f_r = NNR / NNR_0$$) drives the [FCAOR](../tables/capital-fraction-resource-extraction.md) table.

## Calibration

All standard presets use 1.0 (the World3-03 default). No calibration deviation is needed since the NNR depletion trajectory already achieves RMSE = 0.9% against historical estimates at this value. The parameter exists as an exploration lever for sensitivity analysis.

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*. NRI (Non-Renewable Resources Initial) = 1.0 × 10^12 resource units, here normalized to 1.0.
- State initialization: `crates/world3-core/src/model/state.rs`, `WorldState::initial_1900()`.
