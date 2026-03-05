# Model Documentation

Macroco is a system dynamics model based on World3-03 (Meadows et al., 2004 — *Limits to Growth: The 30-Year Update*), as implemented in [pyworld3](https://github.com/cvanwynsberghe/pyworld3). This documentation describes every equation, lookup table, and parameter in the model — what it represents, how it's computed, and where it deviates from the original World3 specification.

As Macroco diverges from World3-03 — through recalibration against modern data (Nebel et al., 2024), structural changes (Delay3 pipelines, dynamic ISOPC), and custom lookup tables — each deviation is documented with rationale, calibration evidence, and references to the original literature.

---

## Sectors

| Sector | State Variables | Tables | Parameters | Description |
|--------|:-:|:-:|:-:|-------------|
| [Population](sectors/population.md) | 8 | 14 | 3 | Age cohorts, mortality, fertility, perceived life expectancy |
| [Capital](sectors/capital.md) | 2 | 7 | 3 | Industrial and service output, investment allocation |
| [Agriculture](sectors/agriculture.md) | 3 | 9 | 3 | Food production, land dynamics, erosion, fertility |
| [Resources](sectors/resources.md) | 1 | 1 | 2 | Non-renewable resource depletion |
| [Pollution](sectors/pollution.md) | 3 | 3 | 1 | Persistent pollution, generation, assimilation |

## Cross-Cutting

- [Feedback Loops](feedback-loops.md) — The six major loops connecting sectors
- [Solver](solver.md) — RK4 integration, evaluation order, lookup table mechanics

---

## Deviation Summary

<!-- audit:deviation-summary -->

| Status | Count | Tables |
|--------|------:|--------|
| Exact match | 23 | M1, M2, M3, M4, LMF, LMHS2, CMI, FPU, LMP, FM, CMPLE, HSAPC, IFPC1, FIOAS1, ISOPC1, JPICU, LYMC, LYMAP1, UILPC, LFDR, LFRT, FALM, FCAOR1 |
| Intentional deviation | 5 | DCFS, FRSN, FIOACV, FIOAA, PPASR |
| Custom / no reference | 6 | LFP, LERD, LDCO, FRNF, PPGIO, PPGAO |
| **Total** | **34** | |

<!-- /audit:deviation-summary -->

---

## How to Read the Charts

The simulator displays six charts, each tracking a key variable from 1900 to 2100.

### The three presets

| Preset | What it assumes | What happens |
|--------|----------------|--------------|
| **BAU** (Business as Usual) | No policy changes. Current trends continue. | Economy grows, hits resource limits, contracts. Population peaks ~2030 then declines. |
| **Technology** | 4× resource efficiency, 80% pollution control, improved agriculture. No social changes. | Buys time but doesn't prevent overshoot — pollution and food limits catch up. |
| **Stabilized** | Technology improvements + aggressive family planning (95% from 1975), land protection, investment restraint. | Closest to a sustainable trajectory. Population stabilizes, resources last longer. |

### What each chart shows

1. **Population** — Total world population in billions. In BAU, it peaks around 8 billion near 2030 then falls as death rates rise from food shortages and pollution.

2. **Resources Remaining** — Fraction of initial non-renewable resources (oil, minerals, etc.) still available. Falls faster as industrial output grows. When it gets low, the cost of extraction rises sharply.

3. **Food Per Capita** — Kilograms of food produced per person per year. Depends on arable land, agricultural investment, and pollution effects on crop yields.

4. **Industrial Output Per Capita** — Economic output per person ($/person/year). Peaks when the economy is growing faster than population, then falls as resource costs rise.

5. **Pollution Index** — Persistent pollution relative to 1970 levels (1970 = 1.0). Once pollution overwhelms the environment's ability to absorb it, the assimilation time grows exponentially.

6. **Life Expectancy** — Average life expectancy in years, driven by food availability, health services, crowding, and pollution.

---

## What Is This?

In 1972, a team of MIT researchers led by Donella and Dennis Meadows published *The Limits to Growth*. They built a computer model called **World 3** that simulated the interactions between population, industrial output, food production, resource consumption, and pollution from 1900 to 2100.

Their central finding: if nothing changes, the world economy grows until it hits physical limits — resource depletion, pollution buildup, or food shortages — and then contracts sharply. They called this pattern **overshoot and collapse**.

Macroco is a faithful reimplementation of the World 3 model, extended with modern calibration. The equations, lookup tables, and initial conditions are drawn from the published model documentation. You can run the same scenarios the Meadows team explored and see the dynamics for yourself.

### What "overshoot" means

Overshoot happens when a system grows beyond what its environment can sustain, then is forced to contract. Think of it like a bank account: you can spend more than you earn for a while by drawing down savings, but eventually the savings run out and spending must fall. In World 3, the "savings" are non-renewable resources, fertile land, and the atmosphere's ability to absorb pollution.

---

## References

- Meadows, D. H., Meadows, D. L., Randers, J., & Behrens, W. W. III. (1972). *The Limits to Growth*. Universe Books.
- Meadows, D. H., Meadows, D. L., & Randers, J. (1992). *Beyond the Limits*. Chelsea Green.
- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green.
- Forrester, J. W. (1971). *World Dynamics*. Wright-Allen Press.
- Nebel, A., et al. (2024). "Recalibration of limits to growth: An update of the World3 model." *Journal of Industrial Ecology*. DOI: [10.1111/jiec.13442](https://doi.org/10.1111/jiec.13442)
- Nebel, A., et al. (2025). Correction to "Recalibration of limits to growth." DOI: [10.1111/jiec.70042](https://doi.org/10.1111/jiec.70042)
- pyworld3: [github.com/cvanwynsberghe/pyworld3](https://github.com/cvanwynsberghe/pyworld3) — Python implementation faithfully digitized from World3-03 Vensim model.
- WorldDynamics.jl: [github.com/worlddynamics/WorldDynamics.jl](https://github.com/worlddynamics/WorldDynamics.jl) — Julia implementation of World3.
