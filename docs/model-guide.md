# Model Guide — Understanding the World 3 Simulation

This guide explains how the Macroco simulation works. Each section has two tracks:

- **Plain English** — What the variable represents and why it matters
- **Technical Detail** — Governing equations, lookup tables, and calibration constants

---

## What Is This?

In 1972, a team of MIT researchers led by Donella and Dennis Meadows published *The Limits to Growth*. They built a computer model called **World 3** that simulated the interactions between population, industrial output, food production, resource consumption, and pollution from 1900 to 2100.

Their central finding: if nothing changes, the world economy grows until it hits physical limits — resource depletion, pollution buildup, or food shortages — and then contracts sharply. They called this pattern **overshoot and collapse**.

Macroco is a faithful reimplementation of the World 3 model. The equations, lookup tables, and initial conditions are drawn from the published model documentation. You can run the same scenarios the Meadows team explored and see the dynamics for yourself.

### What "overshoot" means

Overshoot happens when a system grows beyond what its environment can sustain, then is forced to contract. Think of it like a bank account: you can spend more than you earn for a while by drawing down savings, but eventually the savings run out and spending must fall. In World 3, the "savings" are non-renewable resources, fertile land, and the atmosphere's ability to absorb pollution.

---

## How to Read the Charts

The simulator displays six charts, each tracking a key variable from 1900 to 2100.

### The three presets

| Preset | What it assumes | What happens |
|--------|----------------|--------------|
| **Collapse** | No policy changes. Current trends continue. | Economy grows, hits resource limits, contracts. Population peaks ~2030 then declines. |
| **Technology** | 4x resource efficiency, 80% pollution control, improved agriculture. No social changes. | Buys time but doesn't prevent overshoot — pollution and food limits catch up. |
| **Stabilized** | Technology improvements + aggressive family planning (95% from 1975), land protection, investment restraint. | Closest to a sustainable trajectory. Population stabilizes, resources last longer. |

### What each chart shows

1. **Population** — Total world population in billions. In Collapse, it peaks around 8 billion near 2030 then falls as death rates rise from food shortages and pollution.

2. **Resources Remaining** — Fraction of initial non-renewable resources (oil, minerals, etc.) still available. Falls faster as industrial output grows. When it gets low, the cost of extraction rises sharply.

3. **Food Per Capita** — Kilograms of food produced per person per year. Depends on arable land, agricultural investment, and pollution effects on crop yields.

4. **Industrial Output Per Capita** — Economic output per person ($/person/year). Peaks when the economy is growing faster than population, then falls as resource costs rise.

5. **Pollution Index** — Persistent pollution relative to 1970 levels (1970 = 1.0). Once pollution overwhelms the environment's ability to absorb it, the assimilation time grows exponentially.

6. **Life Expectancy** — Average life expectancy in years, driven by food availability, health services, crowding, and pollution.

---

## Sector Deep Dives

### Population

#### Plain English

The population sector tracks four age groups (0–14, 15–44, 45–64, 65+). People are born, age through the cohorts, and die. How many children are born depends on desired family size and food availability. How long people live depends on food, health services, crowding, and pollution.

**What makes population grow:** High birth rates (large desired family size, adequate food).

**What makes population shrink:** Declining life expectancy (pollution, food shortages, crowding), falling fertility (rising income, family planning).

**Key feedback loop:** As industrial output rises, desired family size falls (the "demographic transition"). But if the economy collapses from resource depletion, health services deteriorate and death rates rise sharply.

#### Technical Detail

**State variables (4 ODE stocks):**
- `cohort_0_14`, `cohort_15_44`, `cohort_45_64`, `cohort_65_plus`

**Life expectancy** is computed as a product of four lookup-table multipliers on a base value:

```
life_expectancy = 28.0 × LEM_food × LEM_health × LMC × LEM_pollution
```

where:
- `LEM_food` = `life_exp_multiplier_food(food_per_capita / subsistence_food)` — ranges 0→0 to 2→1.2 to 5→1.4
- `LEM_health` = `life_exp_multiplier_health(EHSPC)` — ranges 0→1.0 to 100→2.0. EHSPC is a 20-year first-order smooth of HSAPC (health services allocations per capita, from SOPC via lookup table).
- `LMC` = `1 - CMI(IOPC) × FPU(POP)` — World3-03 crowding multiplier. CMI goes negative at mid-IOPC (sanitation/infrastructure reduce crowding mortality), so LMC can exceed 1.0.
- `LEM_pollution` = `life_exp_multiplier_pollution(pollution_index)` — ranges 0→1.0 to 80→0.55

**Fertility** is the product of three factors:

```
total_fertility_rate = desired_family_size(iopc) × family_planning_multiplier(fp_efficacy × fp_ramp) × food_fertility_multiplier(food_ratio)
```

The family planning ramp phases in linearly from 1900 to `family_planning_year`.

**Births per year:**

```
fertile_women = cohort_15_44 × 0.5
births_per_year = fertile_women × total_fertility_rate / 30.0
```

**Mortality** uses World3-03 M1–M4 lookup tables (cohort-specific annual mortality rates indexed by life expectancy):

```
deaths_0_14  = cohort_0_14  × M1(life_expectancy)
deaths_15_44 = cohort_15_44 × M2(life_expectancy)
deaths_45_64 = cohort_45_64 × M3(life_expectancy)
deaths_65+   = cohort_65+   × M4(life_expectancy)
```

**Cohort derivatives:**

```
d(cohort_0_14)/dt  = births - aging_0_to_15 - deaths_0_14
d(cohort_15_44)/dt = aging_0_to_15 - aging_15_to_45 - deaths_15_44
d(cohort_45_64)/dt = aging_15_to_45 - aging_45_to_65 - deaths_45_64
d(cohort_65+)/dt   = aging_45_to_65 - deaths_65+
```

Aging rates: `cohort / duration` where durations are 15, 30, 20 years respectively.

**Lookup tables:** `life_exp_multiplier_food`, `life_exp_multiplier_health`, `health_services_per_capita`, `crowding_multiplier_ind`, `fraction_population_urban`, `life_exp_multiplier_pollution`, `desired_family_size`, `family_planning_multiplier`, `food_fertility_multiplier`

**Source:** `crates/world3-core/src/model/sectors/population.rs`

---

### Industrial Capital

#### Plain English

The capital sector models the world economy. Factories, machines, and infrastructure produce goods (industrial output). Some output is reinvested in building more capital, some goes to services (health, education), and some goes to agriculture. Capital wears out over time (depreciation).

**What makes the economy grow:** Investment exceeds depreciation, so capital accumulates. Technology improvements get more output per unit of capital.

**What makes the economy shrink:** As resources deplete, it takes more and more capital just to extract what's left — less is available for productive output. This is the key mechanism of collapse in the Collapse scenario.

**Key feedback loop:** Resource depletion → higher extraction costs → more capital diverted to extraction → less productive output → slower investment → economic contraction.

#### Technical Detail

**State variables (2 ODE stocks):**
- `industrial_capital`, `service_capital`

**Industrial output:**

```
COR_multiplier = capital_output_ratio_resources(fraction_remaining)
ICOR = 3.0 × COR_multiplier
tech_years = max(time - 1970, 0)
tech_multiplier = (1 + technology_growth_rate)^tech_years
capital_for_resources = capital_fraction_resource_extraction(fraction_remaining)
productive_capital = industrial_capital × (1 - clamp(capital_for_resources, 0, 0.95)) × tech_multiplier
industrial_output = productive_capital / ICOR
```

The `capital_output_ratio_resources` table makes capital less productive as resources deplete: at full resources (1.0) the multiplier is 0.50 (efficient); at zero resources it's 4.0 (extremely wasteful). The breakeven point where capital growth stalls is around 65% remaining.

**Service output:**

```
service_output = service_capital / 1.0   (SCOR = 1.0)
```

**Allocation fractions:**

```
frac_to_agriculture = industrial_fraction_to_agriculture(food_per_capita / subsistence_food)
frac_to_services = industrial_fraction_to_services(spc_normalized)
frac_to_consumption = consumption_fraction(iopc)
frac_to_investment = 1 - frac_to_consumption - frac_to_services - frac_to_agriculture  (residual)
```

**Capital dynamics:**

```
d(industrial_capital)/dt = industrial_output × frac_to_investment - industrial_capital × depreciation_rate
d(service_capital)/dt = industrial_output × frac_to_services - service_capital × depreciation_rate
```

**Constants:** ICOR_1970 = 3.0, SCOR_1970 = 1.0, POP_REFERENCE = 3.6e9

**Lookup tables:** `capital_output_ratio_resources`, `capital_fraction_resource_extraction`, `industrial_fraction_to_agriculture`, `industrial_fraction_to_services`

**Source:** `crates/world3-core/src/model/sectors/capital.rs`

---

### Agriculture

#### Plain English

The agriculture sector tracks how much food the world produces. Food comes from arable land multiplied by yield (how much each hectare produces). Yield depends on agricultural inputs (fertilizer, machinery) and pollution effects on crops. New land can be developed, but the best land was developed first — each additional hectare costs more than the last. Land is also lost to erosion.

**What increases food production:** More arable land, more agricultural inputs, better agricultural technology.

**What decreases food production:** Pollution reduces crop yields. Erosion removes productive land. Development of new marginal land has diminishing returns.

**Key feedback loop:** Industrial output → agricultural investment → higher yields → more food. But pollution → lower yields → less food → divert more output to agriculture → less for industry.

#### Technical Detail

**State variables (2 ODE stocks):**
- `arable_land`, `potentially_arable_land`

**Agricultural inputs per hectare:**

```
frac_to_agri = industrial_fraction_to_agriculture(food_per_capita / subsistence_food)
agri_output_total = industrial_output × frac_to_agri
agri_inputs_per_ha = agri_output_total / arable_land
```

**Land yield:**

```
yield_mult_capital = land_yield_multiplier_capital(agri_inputs_per_ha)
yield_mult_pollution = land_yield_multiplier_pollution(pollution_index)
ag_tech_years = max(year - 1960, 0)
ag_tech = agricultural_technology × (1 + agricultural_technology_growth_rate)^ag_tech_years
land_yield = land_fertility × yield_mult_capital × yield_mult_pollution × ag_tech
```

Base fertility is 600 kg/hectare/year (1900 level). The capital multiplier ranges from 1.0 (no inputs) to ~10.0 (maximum inputs). The pollution multiplier falls from 1.0 (pristine) to 0.40 (heavily polluted).

**Agricultural technology growth (Macroco extension):** World3-03 BAU has no time-varying agricultural technology (`lyf=1`, `LYCM=0` in both pyworld3 and Vensim). Real-world agricultural TFP grew ~1%/yr from 1960-2020 (USDA ERS). The `agricultural_technology_growth_rate` parameter (Collapse default: 0.005/yr) represents Green Revolution gains -- improved cultivars, irrigation, precision farming -- not captured by the capital-driven yield multiplier (LYMC). Growth starts from 1960, the onset of semi-dwarf wheat/rice development by Norman Borlaug. This extension follows the same pattern as `technology_growth_rate` for industrial output.

**Food production:**

```
food = arable_land × land_yield
food_per_capita = food / population
```

**Land development:**

```
land_fraction_developed = 1 - potentially_arable_land / 3.2e9
dev_cost = land_development_cost(land_fraction_developed)
development_rate = min(desired_development / 10.0, potentially_arable / 10.0)
```

Total potential arable land is 3.2 billion hectares. Development cost rises exponentially from 100 to 616 as the fraction developed approaches 1.

**Erosion:**

```
erosion_mult = land_erosion_multiplier(land_yield / 600.0)
erosion = arable_land × 0.002 × erosion_mult × (1 - land_protection_fraction)
```

Higher-intensity farming increases erosion. Land protection fraction (0–50%) can reduce it.

**Derivatives:**

```
d(arable_land)/dt = development_rate - erosion_rate
d(potentially_arable_land)/dt = -development_rate
```

**Constants:** LAND_YIELD_1900 = 600.0 kg/ha/yr, TOTAL_POTENTIAL_ARABLE = 3.2e9 ha, LAND_DEVELOPMENT_TIME = 10.0 yr, LAND_EROSION_RATE = 0.002/yr

**Lookup tables:** `land_yield_multiplier_capital`, `land_yield_multiplier_pollution`, `land_erosion_multiplier`, `land_development_cost`, `industrial_fraction_to_agriculture`

**Source:** `crates/world3-core/src/model/sectors/agriculture.rs`

---

### Non-Renewable Resources

#### Plain English

The resources sector is the simplest: the world starts with a fixed stock of non-renewable resources (think oil, coal, metals, minerals) and extracts them to fuel the economy. The more people there are and the richer they are, the faster resources are consumed. A "resource efficiency" parameter represents technology that lets you get more economic value from each unit of resource.

**What depletes resources faster:** Growing population, rising industrial output per capita.

**What slows depletion:** Higher resource efficiency (technology), economic contraction (which is the mechanism of collapse).

**Key feedback loop:** As resources deplete, the capital sector must divert more capital to extraction, leaving less for productive output. This creates a self-reinforcing cycle: less output → less investment → slower growth → but still consuming resources → further depletion.

#### Technical Detail

**State variable (1 ODE stock):**
- `nonrenewable_resources` (normalized, starts at `initial_nnr_fraction`, usually 1.0)

**Resource extraction:**

```
extraction_rate = population × industrial_output_per_capita × 3.0e-15 / resource_efficiency
d(nonrenewable_resources)/dt = -extraction_rate
```

The depletion coefficient (3.0e-15) is calibrated so that at 1970 conditions (population = 3.6e9, IOPC = $500/yr), the extraction rate is 5.4e-3 of total NNR per year. Cumulatively, this depletes about 50% of NNR by 2050 under the Collapse scenario.

**Fraction remaining** (auxiliary):

```
fraction_remaining = clamp(nonrenewable_resources, 0, 1)
```

This fraction feeds into the capital sector's `capital_output_ratio_resources` and `capital_fraction_resource_extraction` tables, creating the key feedback that drives the Collapse scenario.

**Source:** `crates/world3-core/src/model/sectors/resources.rs`

---

### Pollution

#### Plain English

The pollution sector tracks persistent pollution — long-lasting pollutants that accumulate in the environment (think CO2, heavy metals, persistent organic pollutants). Pollution is generated by industry and agriculture, and the environment slowly absorbs it over time. The critical dynamic: once pollution gets high enough, the environment's ability to absorb it degrades, and assimilation time grows dramatically. This creates a tipping point where pollution spirals upward.

**What increases pollution:** Industrial output, agricultural intensification.

**What decreases pollution:** Pollution control technology, reduced industrial/agricultural activity, natural assimilation.

**Key feedback loop:** Rising pollution → longer assimilation time → faster accumulation → even higher pollution. Also: pollution → reduced crop yields → food shortage → higher death rates.

#### Technical Detail

**State variable (1 ODE stock):**
- `persistent_pollution`

**Pollution generation:**

```
iopc_norm = industrial_output_per_capita / 200
agri_norm = agricultural_inputs_per_hectare / 40

gen_industry = industrial_output × 3.0e-13 × pollution_generation_industry(iopc_norm)
gen_agriculture = arable_land × agri_inputs_per_ha × 1.0e-13 × pollution_generation_agriculture(agri_norm)

generation = (gen_industry + gen_agriculture) × (1 - pollution_control)
```

At 1970 conditions: industrial generation ≈ 0.30 index units/yr, agricultural ≈ 0.005 index units/yr.

**Pollution assimilation:**

```
assimilation_time = pollution_assimilation_time(pollution_index)
assimilation = persistent_pollution / assimilation_time
```

The assimilation time table is the key non-linearity: at low pollution (index 0) it's 20 years (fast cleanup); at index 60 it's 480 years (effectively permanent). This creates the tipping-point behavior.

**Pollution index** (auxiliary):

```
pollution_index = max(persistent_pollution, 0)
```

**Derivative:**

```
d(persistent_pollution)/dt = generation - assimilation
```

**Constants:** PPGIO = 3.0e-13 (industrial coefficient), PPGAO = 1.0e-13 (agricultural coefficient)

**Lookup tables:** `pollution_generation_industry`, `pollution_generation_agriculture`, `pollution_assimilation_time`

**Source:** `crates/world3-core/src/model/sectors/pollution.rs`

---

## Key Feedback Loops

The power of World 3 comes from feedback loops that connect the five sectors. Here are the most important ones:

### 1. Resource Depletion → Economic Collapse

```
Resources deplete → Extraction costs rise → More capital diverted to extraction
→ Less productive capital → Lower industrial output → Less investment
→ Capital depreciates faster than it's replaced → Output falls further
```

This is the primary driver of collapse in the Collapse scenario. Even though there are still resources in the ground, the *cost* of extracting them consumes so much capital that the economy cannot sustain itself.

### 2. Pollution → Agricultural Decline → Food Crisis

```
Industrial output generates pollution → Pollution reduces crop yields
→ Less food per capita → More investment diverted to agriculture
→ Less investment in industry → But pollution persists (long assimilation time)
→ Yields continue falling
```

### 3. Demographic Transition (stabilizing)

```
Rising income → Smaller desired family size → Lower birth rate
→ Slower population growth → Less resource pressure
```

This is the one major *negative* (stabilizing) feedback loop. It's why the Technology and Stabilized scenarios perform better — they allow this transition to proceed before collapse.

### 4. Population → Resource Pressure (reinforcing)

```
More people → More consumption → Faster resource depletion
→ Economic contraction → Higher death rates → Population decline
```

### 5. Pollution Tipping Point (reinforcing)

```
Pollution rises → Assimilation time increases → Pollution accumulates faster
→ Even longer assimilation time → Runaway pollution
```

### 6. Food–Population Balance

```
Adequate food → Lower mortality, higher fertility → Population grows
→ More mouths to feed → Lower food per capita → Higher mortality
```

---

## The Solver

### Plain English

The simulation is a set of 21 quantities (population cohorts, delay stages, capital stocks, arable land, resources, pollution) that change over time. At each time step, the model calculates how fast each quantity is changing, then advances them forward.

Think of it like an accountant updating a ledger: at the end of each "year," you check the births, deaths, investment, depreciation, extraction, and pollution, then update all the balances.

### Technical Detail

The model uses a **4th-order Runge-Kutta (RK4)** solver. RK4 is a standard numerical method that evaluates the derivatives at four points within each time step to get an accurate estimate of the change.

The 21 ODE stock variables are packed into a vector via `WorldState::to_vec()`, the RK4 step is computed, and the result is unpacked via `WorldState::from_vec()`. This avoids manual derivative arithmetic on the typed struct.

**Sector evaluation order matters:** resources → capital → agriculture → population → pollution. Each sector uses the latest values from previously computed sectors within the same step.

### Lookup Tables

Non-linear relationships (like "how does pollution affect crop yields?") are encoded as **piecewise-linear lookup tables** — a series of (x, y) breakpoints with linear interpolation between them. There are 33 tables in total, loaded at startup.

Lookup tables come directly from the published World 3 model documentation. They encode empirical relationships that aren't easily expressed as simple equations.

---

## References

- Meadows, D. H., Meadows, D. L., Randers, J., & Behrens, W. W. III. (1972). *The Limits to Growth*. Universe Books.
- Meadows, D. H., Meadows, D. L., & Randers, J. (1992). *Beyond the Limits*. Chelsea Green.
- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green.
- Forrester, J. W. (1971). *World Dynamics*. Wright-Allen Press.
