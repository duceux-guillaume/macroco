# Agriculture Sector

**Source code:** `crates/world3-core/src/model/sectors/agriculture.rs`

## Overview

The agriculture sector models global food production as a function of cultivated land area and yield per hectare. Yield depends on capital inputs (fertilizer, machinery, irrigation), pollution effects on crops, and inherent land fertility. New arable land can be developed from a finite reserve of potentially arable land, but the best land was brought into cultivation first --- each additional hectare costs more than the last. Existing arable land is lost to erosion (intensified by high-yield farming) and to urban-industrial expansion.

The sector captures two critical dynamics of the overshoot-and-collapse trajectory. First, food production can grow rapidly through capital investment (the Green Revolution effect), but pollution feedback eventually degrades yields. Second, land is a non-renewable stock on human timescales: once eroded or paved over, it does not return.

World3-03 applies two multiplicative factors to gross food output that reduce effective food supply. The **Land Fraction Harvested** ($$\mathrm{LFH} = 0.7$$) accounts for the fact that not all arable land is harvested in any given year (fallow, failed crops, multi-cropping inefficiencies). The **Processing Loss** ($$\mathrm{PL} = 0.1$$) represents post-harvest losses in storage, transport, and processing. Together these reduce effective food to $$0.7 \times 0.9 = 0.63$$ of the gross land-yield product.

## State Variables

The agriculture sector maintains five ODE stocks:

| Variable | Symbol | Initial (1900) | Unit | Description |
|----------|--------|-----------------|------|-------------|
| `arable_land` | $$AL$$ | $$0.9 \times 10^9$$ | hectares | Land currently under cultivation |
| `potentially_arable_land` | $$PAL$$ | $$2.3 \times 10^9$$ | hectares | Land that could be developed for agriculture |
| `urban_industrial_land` | $$UIL$$ | $$8.2 \times 10^6$$ | hectares | Land used for cities and industry |
| `land_fertility` | $$LFERT$$ | 600.0 | kg/ha/yr | Inherent soil productivity |
| `food_per_capita_smooth` | $$FPC_s$$ | 230.0 | kg/person/yr | Smoothed food per capita (2-year perception delay) |

The total potential arable land is $$AL + PAL = 3.2 \times 10^9$$ hectares (FAO estimate). Food per capita ($$FPC$$), food production ($$F$$), land yield ($$LY$$), and agricultural inputs per hectare ($$AIPH$$) are auxiliary variables recomputed each time step.

## Governing Equations

### Agricultural inputs per hectare

Industrial output is allocated to agriculture based on food pressure. The allocation fraction uses the ratio of smoothed food per capita to indicated food per capita:

$$\text{food\_ratio} = \frac{FPC_s}{\mathrm{IFPC}(\mathrm{IOPC})}$$

$$\text{frac\_to\_agri} = \mathrm{FIOAA}(\text{food\_ratio})$$

$$\text{agri\_output} = IO \times \text{frac\_to\_agri}$$

$$AIPH = \frac{\text{agri\_output}}{AL}$$

The indicated food per capita table ([IFPC](../tables/indicated-food-per-capita.md)) scales food demand with industrialization: at low IOPC, $$\mathrm{IFPC} \approx 230$$ (subsistence); at high IOPC, $$\mathrm{IFPC}$$ rises to 1250 kg/person/yr. This prevents the zero-allocation trap where high food output drives $$\text{food\_ratio}$$ so high that agricultural investment collapses.

### Land yield

Land yield is the product of soil fertility and three multipliers:

$$LY = LFERT \times \mathrm{LYMC}(AIPH) \times \mathrm{LYMAP}(\text{pollution\_index}) \times \text{agricultural\_technology}$$

where:

- [LYMC](../tables/land-yield-multiplier-capital.md) maps capital inputs to a yield multiplier (1.0 at zero inputs, up to 10.0 at 1000 $/ha/yr)
- [LYMAP](../tables/land-yield-multiplier-pollution.md) degrades yield under pollution stress (1.0 at low pollution, 0.4 at index 30)
- `agricultural_technology` is a scenario parameter (Collapse: 1.0, Technology/Stabilized: 2.0)

Note that $$LFERT$$ replaces the constant base yield of 600 kg/ha/yr used in simpler formulations. As fertility degrades, base yield falls even before capital or pollution effects.

### Food production

$$F = AL \times LY \times \mathrm{LFH} \times (1 - \mathrm{PL})$$

$$FPC = \frac{F}{\text{population}}$$

With $$\mathrm{LFH} = 0.7$$ and $$\mathrm{PL} = 0.1$$, effective food is 63% of gross yield.

### Land development

New arable land is developed from the potentially arable reserve. Development cost rises exponentially as the fraction already developed increases:

$$\text{land\_fraction\_developed} = 1 - \frac{PAL}{3.2 \times 10^9}$$

$$\text{dev\_cost} = \mathrm{LDCO}(\text{land\_fraction\_developed})$$

$$\text{desired\_development} = \frac{IO \times \text{frac\_to\_agri} \times 0.1}{\text{dev\_cost}}$$

$$\text{development\_rate} = \min\!\left(\frac{\text{desired\_development}}{T_d},\; \frac{PAL}{T_d}\right)$$

where $$T_d = 10$$ years is the land development time. The [LDCO](../tables/land-development-cost.md) table gives costs from 100 (no land developed) to 616 (all land developed).

### Urban-industrial land

Urban-industrial land follows a first-order delay converging on per-capita demand:

$$UIL_{\text{desired}} = \mathrm{UILPC}(\mathrm{IOPC}) \times \text{population}$$

$$\frac{dUIL}{dt} = \frac{UIL_{\text{desired}} - UIL}{T_u}$$

where $$T_u = 10$$ years. Growth is constrained to at most 10% of current arable land per year. The [UILPC](../tables/urban-industrial-land-per-capita.md) table maps IOPC to hectares per person.

### Erosion

$$\text{yield\_ratio} = \frac{LY}{600}$$

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\_ratio}) \times (1 - \text{land\_protection\_fraction})$$

The base erosion rate is 0.001/yr (corresponding to a 1000-year land lifetime). The [LERD](../tables/land-erosion-multiplier.md) multiplier amplifies erosion when yield intensity exceeds the inherent level. The `land_protection_fraction` parameter (0--0.5) can reduce erosion.

### Land fertility dynamics

Land fertility changes through degradation (pollution-driven) and regeneration (maintenance-driven):

$$\text{degradation} = LFERT \times \mathrm{LFDR}(\text{pollution\_index})$$

$$\text{food\_ratio}_{\text{FALM}} = \frac{FPC_s}{\text{SFPC}}$$

$$\text{FALM} = \mathrm{FALM}(\text{food\_ratio}_{\text{FALM}})$$

$$\text{regeneration} = \frac{600 - LFERT}{\mathrm{LFRT}(\text{FALM})}$$

$$\frac{dLFERT}{dt} = \text{regeneration} - \text{degradation}$$

The [LFDR](../tables/land-fertility-degradation.md) table gives the degradation rate as a function of pollution. The [FALM](../tables/fraction-land-maintenance.md) table determines how much agricultural output goes to soil maintenance based on perceived food adequacy. The [LFRT](../tables/land-fertility-regeneration-time.md) table converts maintenance effort to regeneration speed.

### Food perception smoothing

Smoothed food per capita tracks actual FPC with a 2-year perception delay (World3-03: FSPD = 2 yr):

$$\frac{dFPC_s}{dt} = \frac{FPC - FPC_s}{2}$$

This smoothed value is used for allocation decisions (FIOAA, FALM), preventing instantaneous over-reaction to food shocks.

### Derivative summary

$$\frac{dAL}{dt} = \text{development\_rate} - \text{erosion} - \max(dUIL/dt,\; 0)$$

$$\frac{dPAL}{dt} = -\text{development\_rate}$$

$$\frac{dUIL}{dt} = \frac{UIL_{\text{desired}} - UIL}{T_u} \quad \text{(constrained)}$$

$$\frac{dLFERT}{dt} = \text{regeneration} - \text{degradation}$$

$$\frac{dFPC_s}{dt} = \frac{FPC - FPC_s}{2}$$

## Feedback Loops

**Pollution-yield loop (balancing).** Industrial output generates pollution (via the pollution sector). Rising pollution degrades crop yields through LYMAP, reducing food production. Lower food triggers higher agricultural allocation (FIOAA), diverting output from industry. This is the central mechanism through which pollution causes agricultural collapse in the Collapse scenario.

**Food-population loop (balancing).** Food per capita affects life expectancy (via the population sector's LMF table) and fertility (via the [FRNF](../tables/food-fertility-multiplier.md) table). When food falls below subsistence, death rates rise and birth rates may fall, reducing population and thus food demand. This loop delays but does not prevent collapse when food production declines.

**Capital-yield loop (reinforcing, then balancing).** Industrial output invested in agriculture raises AIPH, boosting yields through LYMC. Higher food supports population growth and further industrialization. However, LYMC has sharply diminishing returns: the first 40 $/ha/yr triples yield, but going from 400 to 1000 $/ha/yr adds only 45% more.

**Erosion-fertility loop (reinforcing).** Higher yields (from capital inputs) increase erosion through LERD. Erosion removes arable land, concentrating inputs on remaining land, which raises yield ratios further. Meanwhile, pollution degrades fertility (LFDR), reducing base yield and eventually food output.

## Deviations from World3-03

1. **LFH and PL factors.** Our model explicitly includes the Land Fraction Harvested (0.7) and Processing Loss (0.1) multiplicative factors in the food equation, matching World3-03 specification. Some implementations fold these into lookup table calibration.

2. **Land fertility as dynamic base yield.** The land yield equation uses $$LFERT$$ (an ODE stock) rather than a constant 600 kg/ha/yr. This is faithful to World3-03 where $$\text{ly} = \text{lfert} \times \text{lymc} \times \text{lymap}$$.

3. **Custom erosion table (LERD).** pyworld3 uses LLMY (Land Life Multiplier from Yield) tables with a structurally different role. Our LERD table directly maps yield ratio to an erosion multiplier with 9 breakpoints.

4. **Custom development cost table (LDCO).** pyworld3 uses DCPH (Development Cost Per Hectare) indexed by absolute PAL area. Our table uses fraction-developed as the independent variable with an exponential cost curve.

5. **FIOAA with IFPC normalization.** The agricultural allocation fraction uses $$FPC_s / \mathrm{IFPC}(\mathrm{IOPC})$$ rather than $$FPC / \mathrm{SFPC}$$. This dynamic normalization prevents the zero-allocation trap at high IOPC in Technology and Stabilized scenarios while preserving Collapse behavior (where IFPC $$\approx$$ SFPC at low IOPC).

6. **Food perception smoothing (FSPD).** $$FPC_s$$ is an ODE stock (preserved across RK4 stages) rather than an inline smooth, ensuring consistent inter-sector feedback during solver intermediate stages.

7. **Custom food fertility multiplier (FRNF).** pyworld3 uses FCE (Food Consumption Effect) tables with a different functional role. Our FRNF table maps food ratio to a fertility fraction with 5 breakpoints.

## Lookup Tables

| Abbrev. | Name | Input | Output | Match | Doc |
|---------|------|-------|--------|-------|-----|
| LYMC | Land Yield Multiplier from Capital | AIPH [$/ha/yr] | yield multiplier | Exact | [link](../tables/land-yield-multiplier-capital.md) |
| LYMAP | Land Yield Multiplier from Pollution | pollution index | yield multiplier | Exact | [link](../tables/land-yield-multiplier-pollution.md) |
| LERD | Land Erosion Multiplier | yield ratio | erosion multiplier | Custom | [link](../tables/land-erosion-multiplier.md) |
| LDCO | Land Development Cost | fraction developed | cost multiplier | Custom | [link](../tables/land-development-cost.md) |
| UILPC | Urban-Industrial Land Per Capita | IOPC [$/person/yr] | ha/person | Exact | [link](../tables/urban-industrial-land-per-capita.md) |
| LFDR | Land Fertility Degradation | pollution index | degradation rate [yr$$^{-1}$$] | Exact | [link](../tables/land-fertility-degradation.md) |
| LFRT | Land Fertility Regeneration Time | FALM fraction | time [years] | Exact | [link](../tables/land-fertility-regeneration-time.md) |
| FALM | Fraction Land Maintenance | food ratio | maintenance fraction | Exact | [link](../tables/fraction-land-maintenance.md) |
| FRNF | Food Fertility Multiplier | food ratio | fertility fraction | Custom | [link](../tables/food-fertility-multiplier.md) |

## References

- Meadows, D. H., Meadows, D. L., Randers, J., & Behrens, W. W. III. (1972). *The Limits to Growth*. Universe Books.
- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green.
- pyworld3 reference implementation: `https://github.com/cvanwynsberghe/pyworld3`
