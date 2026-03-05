# Capital Sector

**Source code:** `crates/world3-core/src/model/sectors/capital.rs`

## Overview

The capital sector models the world economy as a stock of physical capital — factories, machines, infrastructure — that produces goods measured in constant 1975 US dollars. Industrial output is divided among four competing claims: household consumption, services (health, education), agriculture, and investment in new capital. Investment is the residual after the first three claims are satisfied, which means that under stress the economy starves its own growth to meet immediate needs.

Capital accumulates when gross investment exceeds depreciation and contracts when it does not. Two feedback mechanisms dominate the sector's long-run dynamics. First, as non-renewable resources deplete, an increasing fraction of industrial capital must be diverted to extraction, leaving less for productive output. Second, the capital-output ratio worsens with depletion, so each remaining unit of capital becomes less efficient. Together, these feedbacks are the primary mechanism of economic collapse in the Collapse scenario.

A parallel stock of service capital provides health and education services. Service capital is funded by a fraction of industrial output and depreciates independently. Service output per capita feeds back into the population sector through life expectancy (via health services) and into the capital sector itself through the service allocation fraction.

## State Variables

The sector maintains three ODE stocks:

| Variable | Symbol | Initial value (1900) | Units |
|---|---|---|---|
| Industrial capital | $IC$ | $2.1 \times 10^{11}$ | 1975 USD |
| Service capital | $SC$ | $1.44 \times 10^{11}$ | 1975 USD |
| Perceived IOPC | $DIOPC$ | 43.75 | 1975 USD / person / yr |

Industrial and service capital are the physical stocks. Perceived IOPC (delayed industrial output per capita) is a first-order exponential smooth of actual IOPC with a 20-year social adjustment delay, representing the lag between changes in material living standards and shifts in social norms — particularly desired family size.

## Governing Equations

### Industrial Output

Industrial output is determined by the productive fraction of capital, scaled by a technology multiplier:

$$IO = \frac{IC \cdot (1 - FCAOR) \cdot T_m}{ICOR}$$

where:

- $$ICOR = 3.0$$ is the industrial capital-output ratio (constant, from World3-03),
- $$FCAOR = \text{FCAOR}(FR)$$ is the [fraction of capital allocated to resource extraction](../tables/capital-fraction-resource-extraction.md), looked up from the resource sector's fraction remaining $$FR$$, clamped to $$[0, 0.95]$$,
- $$T_m = (1 + g)^{\max(t - 1970, 0)}$$ is the technology multiplier, where $$g$$ is the [technology growth rate](../parameters/technology-growth-rate.md).

Industrial output per capita is then:

$$IOPC = \frac{IO}{POP}$$

### Service Output

Service output follows a simpler formulation:

$$SO = \frac{SC}{SCOR}$$

where $$SCOR = 1.0$$ is the service capital-output ratio. Service output per capita is $$SOPC = SO / POP$$.

### Allocation Fractions

Industrial output is allocated to four uses. Three are determined by lookup tables; the fourth (investment) is the residual:

$$f_{agr} = \text{FIOAA}\!\left(\frac{FPC_{smooth}}{IFPC(IOPC)}\right)$$

The [fraction to agriculture](../tables/industrial-fraction-to-agriculture.md) depends on the ratio of smoothed food per capita to [indicated food per capita](../tables/indicated-food-per-capita.md). The indicated FPC rises with IOPC, reflecting increasing dietary expectations as societies industrialize.

$$f_{srv} = \text{FIOAS}\!\left(\frac{SOPC}{ISOPC(IOPC)}\right)$$

The [fraction to services](../tables/industrial-fraction-to-services.md) depends on the ratio of actual to [indicated service output per capita](../tables/indicated-service-per-capita.md). Like IFPC, the indicated SOPC scales with IOPC so that service demand rises with income.

$$f_{con} = \text{FIOACV}(IOPC)$$

The [consumption fraction](../tables/consumption-fraction.md) is looked up directly from IOPC.

$$f_{inv} = \max\!\Big(1 - f_{con} - f_{srv} - f_{agr},\; 0\Big)$$

Investment is the residual, clamped to zero. The three allocation fractions are independently determined and may sum to more than 1.0 during collapse (theoretical maximum: 0.70 + 0.30 + 0.40 = 1.40). When this occurs, investment is squeezed to zero — the economy over-allocates to immediate needs while starving capital formation.

### Capital Dynamics

The two capital stocks evolve as:

$$\frac{d(IC)}{dt} = IO \cdot f_{inv} - IC \cdot \delta_i$$

$$\frac{d(SC)}{dt} = IO \cdot f_{srv} - SC \cdot \delta_s$$

where $$\delta_i$$ is the [industrial depreciation rate](../parameters/industrial-depreciation-rate.md) and $$\delta_s$$ is the [service depreciation rate](../parameters/service-depreciation-rate.md).

### Perceived IOPC

Social norms lag behind actual income by approximately 20 years:

$$\frac{d(DIOPC)}{dt} = \frac{IOPC - DIOPC}{SAD}$$

where $$SAD = 20$$ years is the social adjustment delay. The perceived IOPC drives desired family size in the population sector.

## Feedback Loops

### Resource Depletion and Economic Collapse

The dominant positive (reinforcing) feedback loop in the Collapse scenario:

1. Industrial output drives resource consumption (via the resource sector).
2. As resources deplete, $$FCAOR$$ rises — more capital is diverted to extraction.
3. Less productive capital remains, reducing industrial output.
4. Lower output means less investment, slowing capital accumulation.
5. The economy contracts, but resource extraction continues, deepening depletion.

This loop is self-reinforcing: once resource fraction drops below approximately 0.65, the breakeven point where capital growth stalls, decline accelerates. The FCAOR table makes this nonlinear — at full resources ($$FR = 1.0$$) only 5% of capital goes to extraction, but at $$FR = 0.2$$ the fraction rises to 70%.

### Demographic Transition Connection

The capital sector connects to population dynamics through two channels:

- **Service output** funds health services, which raise life expectancy and lower mortality. As the economy collapses, service capital depreciates without replacement, health services decline, and death rates rise.
- **Perceived IOPC** drives desired family size with a 20-year lag. Rising income triggers the demographic transition (fewer children), but the delay means that population continues growing for decades after economic peak.

### Allocation Competition

During stress, the three allocation fractions compete for a shrinking pie. Low food drives up $$f_{agr}$$; poor services drive up $$f_{srv}$$; consumption is relatively inelastic. Investment, as the residual, absorbs all the pressure. This mechanism ensures that economic decline, once started, is difficult to reverse.

## Deviations from World3-03

### FIOACV Smoothing and Cap

The consumption fraction table uses absolute IOPC as its x-axis (rather than the IOPC/IOPCD ratio in World3-03) and is capped at 0.70 instead of 0.83. The original table has a discontinuity between IOPC/IOPCD = 1.0 and 1.2 where consumption jumps from 0.43 to 0.73, creating an "IOPC trap" that stalls industrial growth. Our smoothed table eliminates this discontinuity while keeping the cap consistent with real-world household consumption shares of 55-60% of GDP. See [Consumption Fraction (FIOACV)](../tables/consumption-fraction.md).

### FIOAA Floor and Adjustment

The fraction to agriculture has a 0.005 floor at high food ratio (instead of zero in World3-03), preventing oscillation in the Ecotopia preset where agricultural investment would drop to zero and trigger yield collapse. Values are slightly higher at moderate food ratio to compensate for Land Fraction Harvested and Processing Loss food reduction factors. See [Industrial Fraction to Agriculture (FIOAA)](../tables/industrial-fraction-to-agriculture.md).

### Technology Growth Rate

World3-03 has no technology progress in the Collapse scenario. We add an annual TFP improvement of 1.4% post-1970 to match the historical IOPC trajectory, since the original 1972 model did not anticipate real-world productivity gains of approximately 1.5% per year. See [Technology Growth Rate](../parameters/technology-growth-rate.md).

### Industrial Depreciation Rate

World3-03 uses $$alic_1 = 14$$ years (depreciation rate $$1/14 \approx 0.0714$$). We use $$1/13 \approx 0.0769$$ to reduce early IOPC overshoot caused by our model's structural differences in population dynamics. See [Industrial Depreciation Rate](../parameters/industrial-depreciation-rate.md).

## Lookup Tables

| Table | Abbreviation | Status | Description |
|---|---|---|---|
| [Consumption Fraction](../tables/consumption-fraction.md) | FIOACV | Intentional deviation | Fraction of IO to household consumption |
| [Industrial Fraction to Agriculture](../tables/industrial-fraction-to-agriculture.md) | FIOAA | Intentional deviation | Fraction of IO to agricultural investment |
| [Indicated Food Per Capita](../tables/indicated-food-per-capita.md) | IFPC | Exact match | Desired food level as function of IOPC |
| [Industrial Fraction to Services](../tables/industrial-fraction-to-services.md) | FIOAS | Exact match | Fraction of IO to service investment |
| [Indicated Service Per Capita](../tables/indicated-service-per-capita.md) | ISOPC | Exact match | Desired service level as function of IOPC |
| [Jobs Per Capital Unit](../tables/jobs-per-capital.md) | JPICU | Exact match | Labor intensity of industrial capital |
| [Labor Force Participation](../tables/labor-force-participation.md) | LFP | Custom | Labor participation by age structure |

## Info Panel

### capital.perceived_iopc

**Name:** Perceived Income

**Unit:** $/person/yr (1975)

**Stock:** true

**Beginner:** What people perceive as their standard of living, lagging behind actual income by about 20 years. Social norms and family size expectations adjust to this delayed perception, not to current income.

**Expert:** First-order delay of IOPC with time constant 20 years (World3-03: DIOPC = Smooth(IOPC, SAD=20yr)). Drives desired family size lookup (SFSN) -- social norms adjust slowly to income changes.

**Feedback loops:** demographic-transition

**Related variables:** capital.industrial_output_per_capita, population.fertility_rate

### capital.industrial_capital

**Name:** Industrial Capital

**Unit:** USD (1975)

**Stock:** true

**Beginner:** The total stock of factories, machines, and infrastructure. Grows when investment exceeds wear-and-tear (depreciation). Produces the industrial output that drives the economy.

**Expert:** d(IC)/dt = industrial_output x frac_to_investment - IC x depreciation_rate. Investment is the residual after consumption, services, and agriculture allocation.

**Feedback loops:** resource-collapse, demographic-transition

**Related variables:** capital.industrial_output, capital.industrial_output_per_capita, resources.fraction_remaining

### capital.industrial_output

**Name:** Industrial Output

**Unit:** USD/yr (1975)

**Stock:** false

**Beginner:** Total economic production per year. Split between investment (building more capital), services (health, education), and agriculture.

**Expert:** IO = productive_capital / ICOR. productive_capital = IC x (1 - capital_for_resources) x tech_multiplier. ICOR = 3.0 x COR_resources(fraction_remaining).

**Feedback loops:** resource-collapse, pollution-food

**Related variables:** capital.industrial_capital, capital.industrial_output_per_capita

### capital.industrial_output_per_capita

**Name:** Industrial Output Per Capita

**Unit:** $/person/yr (1975)

**Stock:** false

**Beginner:** Economic output divided by population -- a rough measure of average income. When it rises, people choose smaller families. When it falls, the economy is contracting.

**Expert:** IOPC = industrial_output / population. Drives desired_family_size lookup and food_ratio calculations. Key indicator of economic health.

**Feedback loops:** resource-collapse, demographic-transition

**Related variables:** capital.industrial_output, population.population, population.fertility_rate

### capital.service_output_per_capita

**Name:** Service Output Per Capita

**Unit:** $/person/yr (1975)

**Stock:** false

**Beginner:** How much is spent on services (healthcare, education) per person. Higher service output improves life expectancy through better health services.

**Expert:** SOPC = service_capital / SCOR / population. Feeds into health services calculation and service allocation fraction.

**Feedback loops:** food-population

**Related variables:** population.life_expectancy

## References

- Meadows, D. H., Meadows, D. L., Randers, J., & Behrens, W. W. (1972). *The Limits to Growth*. Universe Books.
- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference implementation: https://github.com/cvanwynsberghe/pyworld3
