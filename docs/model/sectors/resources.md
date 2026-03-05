# Non-Renewable Resources Sector

**Source code:** `crates/world3-core/src/model/sectors/resources.rs`

## Overview

The resources sector is the simplest of the five World3 sectors. It models a single, finite stock of non-renewable resources — an aggregate representing fossil fuels, metals, and minerals — that is drawn down monotonically by industrial activity. There is no replenishment: once extracted, resources are gone.

The sector's importance lies not in its own dynamics but in the feedback it imposes on the capital sector. As the resource stock depletes, an increasing fraction of industrial capital must be diverted to extraction (mining deeper deposits, processing lower-grade ores). This diversion reduces productive output, which in turn reduces investment, initiating the self-reinforcing collapse loop that defines the Collapse trajectory.

## State Variables

The sector contains a single ODE stock:

| Variable | Symbol | Units | Initial value (1900) |
|---|---|---|---|
| `nonrenewable_resources` | $$NNR$$ | dimensionless (normalized) | 1.0 |

The stock is normalized so that $$NNR = 1.0$$ represents the full initial endowment. This normalization avoids the need to specify absolute resource quantities in physical units — all dynamics depend on the *fraction remaining*, not the absolute level.

An auxiliary variable, `fraction_remaining`, is derived by clamping the stock to $$[0, 1]$$:

$$f_r = \text{clamp}(NNR, 0, 1)$$

This fraction is consumed by the capital sector's FCAOR lookup table to determine how much capital is diverted to resource extraction.

## Governing Equations

Resource extraction is driven by two factors: population and industrial output per capita (IOPC). More people consuming more goods per person means faster depletion. A `resource_efficiency` parameter represents technology that reduces resource use per unit of output.

$$\frac{d(NNR)}{dt} = -\frac{P \times IOPC \times k}{r_e}$$

where:

- $$P$$ is total population (persons),
- $$IOPC$$ is industrial output per capita (1975 USD/person/year), clamped to $$\geq 0$$,
- $$k = 3.0 \times 10^{-15}$$ is the resource depletion coefficient (NNR fraction per person per USD per year),
- $$r_e$$ is the `resource_efficiency` parameter (dimensionless multiplier, $$\geq 1$$).

The depletion coefficient $$k$$ is calibrated so that at 1970 conditions ($$P = 3.6 \times 10^9$$, $$IOPC \approx 500$$ USD/yr), the extraction rate is approximately $$5.4 \times 10^{-3}$$ of total NNR per year. Cumulatively, this depletes roughly 50% of NNR by 2050 under Collapse.

The derivative is always non-positive. If population is zero or negative, extraction is zero.

In the source code, the constant is named `RESOURCE_DEPLETION_COEFF` and has the value `0.3e-14` (i.e., $$3.0 \times 10^{-15}$$).

## Feedback Loops

The resources sector participates in one critical feedback loop — the **resource-capital diversion loop** — which operates through the capital sector:

1. Industrial activity extracts resources, reducing $$NNR$$.
2. As $$f_r$$ falls, the FCAOR table (`capital_fraction_resource_extraction`) returns a higher fraction of capital diverted to extraction.
3. Less capital remains for productive industrial output.
4. Lower industrial output reduces IOPC.
5. Lower IOPC slows extraction (a weak negative feedback), but also reduces investment in new capital.
6. The net effect is self-reinforcing: less productive capital means less output, less investment, and eventual economic contraction — even though extraction itself slows.

This is the primary mechanism of Collapse-scenario collapse. When resources fall below roughly 50% of initial stock, the capital diversion begins to dominate, and industrial output peaks and then declines irreversibly.

Note that resource scarcity feeds back *only* through the FCAOR fraction in the capital sector. It does not directly affect the incremental capital-output ratio (ICOR), which remains constant. This is faithful to the World3-03 specification.

## Deviations from World3-03

**`resource_efficiency` > 1.0 in Collapse.** The original World3-03 model uses $$r_e = 1.0$$ for the standard run (no efficiency improvement). Our Collapse preset uses $$r_e = 1.05$$ to compensate for real-world extraction efficiency gains over the 20th and early 21st centuries that the 1972 model did not anticipate. This slight increase delays NNR depletion just enough to improve historical calibration against World Bank data (NNR RMSE = 0.9%) without altering the qualitative overshoot-and-collapse trajectory.

The Technology and Ecotopia presets use $$r_e = 4.0$$, representing aggressive efficiency gains that substantially delay resource depletion.

## Lookup Tables

The sector itself contains no lookup tables — extraction depends only on population, IOPC, and the depletion coefficient. However, the sector's output ($$f_r$$) feeds into one table in the capital sector:

| Table | World3-03 name | Input | Output | Status |
|---|---|---|---|---|
| [`capital_fraction_resource_extraction`](../tables/capital-fraction-resource-extraction.md) | FCAOR1 | $$f_r$$ (fraction remaining) | fraction of capital diverted to extraction | Exact match |

See the [FCAOR table documentation](../tables/capital-fraction-resource-extraction.md) for breakpoints and interpretation.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing. Chapter on non-renewable resources.
- Meadows, D. H., Meadows, D. L., Randers, J., & Behrens, W. W. (1972). *The Limits to Growth*. Universe Books. Figure 35 (standard run).
- pyworld3 reference implementation: `pyworld3/capital.py` (FCAOR evaluation), `pyworld3/functions_table_world3.json` (FCAOR1 breakpoints).
