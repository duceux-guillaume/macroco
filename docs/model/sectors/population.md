# Population Sector

**Source code:** `crates/world3-core/src/model/sectors/population.rs`

## Overview

The population sector tracks four age cohorts: children (0--14), young adults (15--44), middle-aged adults (45--64), and seniors (65+). People are born into the youngest cohort, age through successively older cohorts at fixed durations, and die at rates that depend on life expectancy. The number of births depends on desired family size, compensatory fertility expectations, family planning programs, food availability, and biological fecundity limits. The number of deaths depends on life expectancy, which is itself a product of four factors: food adequacy, health services, crowding, and environmental pollution.

What makes population grow is high birth rates --- large desired family size when industrial output per capita is moderate, adequate food supply, and low perceived life expectancy (which triggers compensatory fertility). What makes population shrink is declining life expectancy from pollution, food shortages, or crowding, combined with falling fertility as rising income drives the demographic transition.

The critical feedback loop in this sector is the demographic transition: as industrial output rises, desired family size falls, slowing population growth. But this transition operates with a 20-year social perception delay (perceived IOPC) --- families adjust their fertility expectations slowly. If the economy collapses from resource depletion before the transition completes, health services deteriorate, death rates rise sharply, and population declines through elevated mortality rather than voluntary fertility reduction.

## State Variables

| Variable | Description | Initial (1900) | Units | Source |
|----------|-------------|---------------:|-------|--------|
| `cohort_0_14` | Population aged 0--14 | 6.50 x 10^8 | persons | World3-03 p1i |
| `cohort_15_44` | Population aged 15--44 | 7.00 x 10^8 | persons | World3-03 p2i |
| `cohort_45_64` | Population aged 45--64 | 1.90 x 10^8 | persons | World3-03 p3i |
| `cohort_65_plus` | Population aged 65+ | 6.00 x 10^7 | persons | World3-03 p4i |
| `perceived_le` | Perceived life expectancy (Delay3 output) | 33.0 | years | Matches 1900 computed LE |
| `perceived_le_stage1` | Delay3 pipeline stage 1 | 33.0 | years | Steady-state at 1900 LE |
| `perceived_le_stage2` | Delay3 pipeline stage 2 | 33.0 | years | Steady-state at 1900 LE |
| `ehspc` | Effective health services per capita (20-yr smooth) | 7.2 | USD/person/yr | HSAPC(SOPC=90) at 1900 |

Total initial population is 1.6 billion, the sum of the four cohorts.

## Governing Equations

### Life Expectancy

Life expectancy is computed as a product of four lookup-table multipliers on a base value representing subsistence-level longevity:

$$
\text{LE} = \text{LE}_{\text{base}} \times \text{LMF}(f_r) \times \text{LMHS}(\text{EHSPC}) \times \text{LMC} \times \text{LMP}(P_{\text{idx}})
$$

where $\text{LE}_{\text{base}} = 28$ years (World3-03 LEN), $f_r = \text{FPC} / \text{SFPC}$ is the food ratio, and $P_{\text{idx}}$ is the pollution index.

The four multipliers are:

- [Life expectancy multiplier from food](../tables/life-exp-multiplier-food.md) (LMF): ranges from 0 at starvation to 1.4 at food abundance.
- [Life expectancy multiplier from health services](../tables/life-exp-multiplier-health.md) (LMHS2): input is EHSPC (effective health services per capita), a 20-year first-order smooth of HSAPC. Ranges from 1.0 (no services) to 2.0 (modern medicine).
- Crowding multiplier from life conditions (LMC): a composite of two tables:

$$
\text{LMC} = \max\!\bigl(0,\; 1 - \text{CMI}(\text{IOPC}) \times \text{FPU}(\text{POP})\bigr)
$$

where [CMI](../tables/crowding-multiplier-ind.md) is the crowding multiplier from industrialization and [FPU](../tables/fraction-population-urban.md) is the fraction of population urban. CMI goes negative at mid-IOPC (sanitation and infrastructure reduce crowding mortality), so LMC can exceed 1.0.

- [Life expectancy multiplier from pollution](../tables/life-exp-multiplier-pollution.md) (LMP): ranges from 1.0 at no pollution to 0.2 at extreme pollution.

The computed life expectancy is clamped to the range [5, 90] years.

#### Effective Health Services Per Capita

Health services per capita is mapped from service output per capita through the [HSAPC](../tables/health-services-per-capita.md) lookup table, then scaled by the [health investment multiplier](../parameters/health-investment-multiplier.md):

$$
\text{HSAPC} = \text{HSAPC\_table}(\text{SOPC}) \times h_{\text{mult}}
$$

The effective value is a 20-year first-order exponential smooth (World3-03 HSID = 20 years):

$$
\frac{d(\text{EHSPC})}{dt} = \frac{\text{HSAPC} - \text{EHSPC}}{20}
$$

### Fertility

Total fertility rate is the minimum of desired fertility and biological maximum:

$$
\text{TFR} = \min\!\bigl(\text{TFR}_{\text{desired}},\; \text{MTF} \times \text{FM}(\text{LE})\bigr)
$$

where $\text{MTF} = 12$ children/woman (World3-03 biological ceiling) and [FM](../tables/fecundity-multiplier.md) is the fecundity multiplier from life expectancy.

Desired fertility combines four factors:

$$
\text{TFR}_{\text{desired}} = \text{DCFS}(\text{DIOPC}) \times \text{CMPLE}(\text{PLE}) \times \text{FRSN}(e_{\text{fp}}) \times \text{FFM}(f_r)
$$

- [Desired completed family size](../tables/desired-family-size.md) (DCFS): function of perceived industrial output per capita (DIOPC), which is a 20-year smooth of IOPC capturing the social adjustment lag in fertility expectations.
- [Compensatory multiplier from perceived LE](../tables/compensatory-fertility.md) (CMPLE): when perceived life expectancy is low (high infant mortality), women have more children to compensate for expected deaths.
- [Family planning multiplier](../tables/family-planning-multiplier.md) (FRSN): effect of family planning programs. The effective family planning input is:

$$
e_{\text{fp}} = e_{\text{max}} \times \text{ramp}(t)
$$

where $e_{\text{max}}$ is the [family planning efficacy](../parameters/family-planning-efficacy.md) parameter and the ramp phases in linearly from zero at 1900 to full efficacy at [family planning year](../parameters/family-planning-year.md).

- Food fertility multiplier (FFM): effect of food availability on fertility (from Agriculture sector tables).

### Mortality

Mortality uses World3-03 cohort-specific lookup tables indexed by life expectancy. Each table gives the annual mortality rate for one age cohort:

$$
D_i = C_i \times M_i(\text{LE})
$$

where $C_i$ is cohort population and $M_i$ is the mortality rate table:

- [M1](../tables/mortality-0-14.md): mortality rate for ages 0--14
- [M2](../tables/mortality-15-44.md): mortality rate for ages 15--44
- [M3](../tables/mortality-45-64.md): mortality rate for ages 45--64
- [M4](../tables/mortality-65-plus.md): mortality rate for ages 65+

### Cohort Dynamics

People age through cohorts at fixed rates determined by cohort duration. Aging rate is simply cohort population divided by time spent in that cohort:

$$
\frac{dC_{0\text{--}14}}{dt} = B - \frac{C_{0\text{--}14}}{15} - D_{0\text{--}14}
$$

$$
\frac{dC_{15\text{--}44}}{dt} = \frac{C_{0\text{--}14}}{15} - \frac{C_{15\text{--}44}}{30} - D_{15\text{--}44}
$$

$$
\frac{dC_{45\text{--}64}}{dt} = \frac{C_{15\text{--}44}}{30} - \frac{C_{45\text{--}64}}{20} - D_{45\text{--}64}
$$

$$
\frac{dC_{65+}}{dt} = \frac{C_{45\text{--}64}}{20} - D_{65+}
$$

Births enter the youngest cohort:

$$
B = \frac{C_{15\text{--}44} \times 0.5 \times \text{TFR}}{30}
$$

where the factor of 0.5 estimates the female fraction of the fertile-age cohort, and division by 30 converts total fertility rate (children per woman over a lifetime) to annual births per woman.

### Perceived Life Expectancy

Perceived life expectancy is implemented as a Delay3 (three cascaded first-order stages), matching the World3-03 specification (LPD = 20 years). Each stage has time constant $\tau = 20/3$ years:

$$
\frac{d S_1}{dt} = \frac{\text{LE} - S_1}{\tau}
$$

$$
\frac{d S_2}{dt} = \frac{S_1 - S_2}{\tau}
$$

$$
\frac{d(\text{PLE})}{dt} = \frac{S_2 - \text{PLE}}{\tau}
$$

The Delay3 pipeline produces more uniform transit time behavior than a single first-order delay (Delay1), meaning perceived LE responds to changes in actual LE with a sharper, more realistic transition.

## Feedback Loops

**Demographic transition (negative feedback):** Rising industrial output per capita increases DIOPC (with a 20-year lag), which reduces desired family size through the DCFS table. Lower desired family size reduces births, slowing population growth. This is the fundamental mechanism by which economic development stabilizes population.

**Compensatory fertility (positive feedback):** Low perceived life expectancy (from poor health services or high mortality) increases the compensatory multiplier CMPLE, causing women to have more children. This partially offsets mortality improvements --- even as death rates fall, birth rates remain elevated until perceived LE catches up (20-year Delay3).

**Health services feedback (negative on mortality):** Industrial output funds service capital, which produces service output per capita. SOPC maps through HSAPC to health spending, which (after 20-year EHSPC smoothing) increases life expectancy via LMHS. Higher LE reduces mortality, growing population. But growing population dilutes IOPC and SOPC, potentially limiting health investment.

**Pollution-mortality feedback (positive, delayed):** Industrial and agricultural output generate persistent pollution (see [Pollution sector](pollution.md)), which reduces life expectancy through LMP. The pollution appearance delay (20-year Delay3) means mortality effects lag the pollution generation by decades.

**Food-population feedback:** Population growth increases food demand. If food per capita falls below subsistence, LMF drops sharply, collapsing life expectancy and increasing mortality. Simultaneously, low food ratio increases FFM, slightly boosting fertility --- a destabilizing response to food stress. See [Agriculture sector](agriculture.md).

**Crowding feedback:** As population grows, the urban fraction rises (FPU), increasing the crowding mortality term. At low IOPC, CMI is strongly positive (crowding kills), but at moderate IOPC, CMI goes negative (infrastructure benefits outweigh crowding), making LMC > 1.0.

## Deviations from World3-03

**Delay3 for perceived life expectancy.** World3-03 specifies PLE = Delay3(LE, LPD=20). The pyworld3 implementation uses a single first-order delay (Delay1), which has different transient behavior --- exponential approach rather than the S-shaped response of a third-order pipeline. Our implementation uses the full Delay3 (three cascaded first-order stages with $\tau = 20/3$), adding two intermediate ODE stocks (`perceived_le_stage1`, `perceived_le_stage2`) to WorldState.

**Merged DCFS table.** World3-03 computes desired completed family size as $\text{DCFS} = \text{dcfsn} \times \text{SFSN}(\text{DIOPC})$, where dcfsn = 3.8 is a constant and SFSN is a social family size norm lookup. Our implementation merges these into a single [DCFS](../tables/desired-family-size.md) lookup table with intentionally different values, calibrated for the Delay3 perceived-LE dynamics. The pyworld3 effective DCFS values (dcfsn x SFSN) caused a 14.15 billion population spike (79.5% RMSE) when combined with our Delay3 implementation. See the table file for detailed deviation rationale.

**Single LMHS table.** World3-03 has two health services multiplier tables: LMHS1 (pre-1940, lower ceiling of 1.8) and LMHS2 (post-1940, ceiling of 2.0). We use only [LMHS2](../tables/life-exp-multiplier-health.md), which reflects the post-1940 medical technology regime applicable to the model's calibration period.

**Family planning multiplier axis.** The [FRSN](../tables/family-planning-multiplier.md) table uses a 0--1 effectiveness scale (where 0 = no family planning, 1 = maximum effectiveness) rather than World3-03's family income expectation difference axis (-0.2 to 0.2). The mechanism differs but produces comparable net effects on fertility at Collapse settings.

## Lookup Tables

| Table | World3-03 Name | Role | Status |
|-------|---------------|------|--------|
| [Life expectancy multiplier from food](../tables/life-exp-multiplier-food.md) | LMF / LMFT | Food adequacy effect on LE | Exact match |
| [Life expectancy multiplier from health services](../tables/life-exp-multiplier-health.md) | LMHS2 | Health services effect on LE | Exact match (with LMHS2) |
| [Crowding multiplier from industrialization](../tables/crowding-multiplier-ind.md) | CMI | Crowding/infrastructure effect on mortality | Exact match |
| [Fraction of population urban](../tables/fraction-population-urban.md) | FPU | Urban fraction for crowding calculation | Exact match |
| [Life expectancy multiplier from pollution](../tables/life-exp-multiplier-pollution.md) | LMP / LMPDE | Pollution effect on LE | Exact match |
| [Mortality 0--14](../tables/mortality-0-14.md) | M1 | Cohort mortality rate, ages 0--14 | Exact match |
| [Mortality 15--44](../tables/mortality-15-44.md) | M2 | Cohort mortality rate, ages 15--44 | Exact match |
| [Mortality 45--64](../tables/mortality-45-64.md) | M3 | Cohort mortality rate, ages 45--64 | Exact match |
| [Mortality 65+](../tables/mortality-65-plus.md) | M4 | Cohort mortality rate, ages 65+ | Exact match |
| [Desired completed family size](../tables/desired-family-size.md) | DCFS (dcfsn x SFSN) | Income-driven fertility preference | Intentional deviation |
| [Family planning multiplier](../tables/family-planning-multiplier.md) | FRSN | Family planning program effect | Intentional deviation (structural) |
| [Fecundity multiplier](../tables/fecundity-multiplier.md) | FM | Biological fertility ceiling from health | Exact match |
| [Compensatory fertility](../tables/compensatory-fertility.md) | CMPLE | Infant mortality compensation | Exact match |
| [Health services per capita](../tables/health-services-per-capita.md) | HSAPC | Service output to health spending | Exact match (aligned) |

## Info Panel

### population.population

**Name:** World Population

**Unit:** persons

**Stock:** true

**Beginner:** Total number of people alive on Earth. Grows when births exceed deaths, shrinks when deaths exceed births.

**Expert:** Sum of four age cohorts (0--14, 15--44, 45--64, 65+). Each cohort is an ODE stock with inflows (births or aging-in) and outflows (aging-out or deaths). Mortality is age-weighted: base_mort = 1/life_expectancy, with multipliers 0.8, 0.5, 1.0, 3.0 for the four cohorts.

**Feedback loops:** demographic-transition, population-resource, food-population

**Related variables:** population.life_expectancy, population.birth_rate, population.death_rate, population.fertility_rate

### population.life_expectancy

**Name:** Life Expectancy

**Unit:** years

**Stock:** false

**Beginner:** How long the average person lives. Depends on food availability, healthcare, crowding, and pollution. When conditions deteriorate, life expectancy falls and death rates rise.

**Expert:** life_expectancy = 28.0 x LEM_food(food_ratio) x LEM_health(HSAPC) x LMC(CMI,FPU) x LEM_pollution(pollution_index). Product of four lookup-table multipliers on a 28-year base. Health services per capita from World3-03 HSAPC table (maps SOPC to health spending directly).

**Feedback loops:** food-population, pollution-food

**Related variables:** population.population, agriculture.food_per_capita, pollution.pollution_index

### population.birth_rate

**Name:** Birth Rate

**Unit:** per year

**Stock:** false

**Beginner:** How many babies are born per person per year. Falls as income rises (people choose smaller families) and with family planning programs.

**Expert:** birth_rate = births_per_year / population. births_per_year = (cohort_15_44 x 0.5) x total_fertility_rate / 30.0. TFR = desired_family_size(iopc) x fp_multiplier(efficacy x ramp) x food_fertility(food_ratio).

**Feedback loops:** demographic-transition, food-population

**Related variables:** population.population, population.fertility_rate

### population.death_rate

**Name:** Death Rate

**Unit:** per year

**Stock:** false

**Beginner:** How many people die per person per year. Rises when food, healthcare, or environmental conditions worsen.

**Expert:** death_rate = total_deaths / population. Deaths per cohort use age-weighted base mortality (1/life_expectancy x age_factor).

**Feedback loops:** food-population, pollution-food

**Related variables:** population.population, population.life_expectancy

### population.fertility_rate

**Name:** Total Fertility Rate

**Unit:** children/woman

**Stock:** false

**Beginner:** Average number of children per woman. Drops as income rises and family planning becomes available. Below ~2.1, population eventually declines.

**Expert:** TFR = desired_family_size(iopc) x family_planning_multiplier(efficacy x ramp) x food_fertility_multiplier(food_ratio). Desired family size ranges from 5.0 at iopc=0 to 1.9 at iopc=1600.

**Feedback loops:** demographic-transition

**Related variables:** population.population, capital.industrial_output_per_capita

### population.perceived_le

**Name:** Perceived Life Expectancy

**Unit:** years

**Stock:** true

**Beginner:** What people believe the average lifespan to be, based on recent experience. Lags behind actual life expectancy by about 20 years. Drives family size decisions -- when parents perceive high child mortality, they have more children as compensation.

**Expert:** First-order delay of actual LE with time constant 20 years (World3-03: PLE = Smooth(LE, LPD=20yr), simplified from Delay3). Feeds CMPLE lookup: at low perceived LE, compensatory fertility multiplier > 1.

**Feedback loops:** demographic-transition

**Related variables:** population.life_expectancy, population.fertility_rate

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing. Chapter 3: Population sector structure.
- Nebel, A., Kling, A., Willamowski, R., & Vanwynsberghe, C. (2024). pyworld3: A Python implementation of the World3 model. *GitHub*. `https://github.com/cvanwynsberghe/pyworld3`
