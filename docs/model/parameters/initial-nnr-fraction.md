# Initial NNR Fraction (`initial_nnr_fraction`)

**Source code:** `crates/world3-core/src/model/params.rs`

**Sector:** Resources

**Units:** dimensionless fraction

**Range:** 0.25 -- 2.0 (UI slider step: 0.25)

## Values by Preset

| Preset | Value |
|---|---|
| BAU (Collapse) | 1.0 |
| Technology (Technotopia) | 1.0 |
| Stabilized (Ecotopia) | 1.0 |
| World3-03 original | 1.0 |

## Equation Context

The initial NNR fraction sets the starting value of the non-renewable resources ODE stock at year 1900:

$$NNR(1900) = \texttt{initial\_nnr\_fraction}$$

The stock then evolves according to the depletion equation: $$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$. The fraction remaining ($$f_r = NNR / NNR_0$$) drives the [FCAOR](../tables/capital-fraction-resource-extraction.md) table.

## Calibration

All standard presets use 1.0 (the World3-03 default). No calibration deviation is needed since the NNR depletion trajectory already achieves RMSE = 0.9% against historical estimates at this value. The parameter exists as an exploration lever for sensitivity analysis.

## Role in the Model

The initial NNR fraction sets the starting level of the non-renewable resources stock at the beginning of the simulation (year 1900). A value of 1.0 represents the full initial endowment — the conventional World3 assumption that 100% of extractable resources remain at the start of the industrial era.

The parameter is used to initialize the `nonrenewable_resources` ODE stock in `WorldState::initial_1900()`:

$$NNR(1900) = \texttt{initial\_nnr\_fraction}$$

Since the resources sector equation is:

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$

a higher initial stock simply means more resource to deplete before the FCAOR feedback begins constraining the economy. The extraction *rate* at any given moment depends only on current population, IOPC, and efficiency — not on the stock level. The stock level matters because it determines when the fraction remaining ($$f_r$$) drops low enough to trigger capital diversion through the FCAOR table.

## Use Cases

All standard presets use $$\texttt{initial\_nnr\_fraction} = 1.0$$. The parameter exists as a scenario lever for exploratory analysis:

- **$$< 1.0$$ (scarcer world):** Simulates a world with fewer accessible resources — perhaps due to geological constraints or prior extraction. Collapse arrives earlier because the FCAOR threshold is reached sooner.
- **$$> 1.0$$ (more abundant world):** Simulates discovery of additional resource reserves or access to previously uneconomical deposits. The slider allows up to 2.0 (double the standard endowment), which delays but does not prevent BAU collapse — the exponential growth dynamics eventually overwhelm any finite stock.
- **$$= 1.0$$ (standard):** The World3 baseline assumption. Reproduces the original Meadows trajectory.

## Interaction with Other Parameters

The timing of resource-driven collapse depends on both the initial stock and the extraction rate. Two parameter combinations can produce similar trajectories:

- Doubling $$\texttt{initial\_nnr\_fraction}$$ from 1.0 to 2.0 delays 50% depletion by several decades.
- Doubling $$\texttt{resource\_efficiency}$$ from 1.0 to 2.0 halves the extraction rate, producing a similar delay.

However, the effects are not identical in shape. A larger initial stock extends the "abundance" phase (FCAOR = 0.05) without changing the steepness of the transition once depletion begins. Higher efficiency, by contrast, permanently reduces the extraction rate, stretching the entire depletion curve.

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*. NRI (Non-Renewable Resources Initial) = 1.0 × 10^12 resource units, here normalized to 1.0.
- State initialization: `crates/world3-core/src/model/state.rs`, `WorldState::initial_1900()`.
