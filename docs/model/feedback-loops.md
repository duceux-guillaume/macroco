# Feedback Loops

The power of World 3 comes from feedback loops that connect the five sectors. These loops --- some reinforcing (amplifying change), some balancing (resisting change) --- determine whether the system grows, stabilizes, or collapses.

A reinforcing loop drives exponential change: more of A causes more of B, which causes more of A. A balancing loop resists change: more of A causes more of B, which causes *less* of A. The interplay of these loops, and the *delays* between cause and effect, produces the overshoot-and-collapse dynamics that define the standard run.


## 1. Resource Depletion --> Economic Collapse (reinforcing)

**Connects:** [Resources](sectors/resources.md) --> [Capital](sectors/capital.md)

This is the primary driver of collapse in the Collapse scenario. Even though there are still resources in the ground, the *cost* of extracting them consumes so much capital that the economy cannot sustain itself.

```
Resources deplete --> Extraction costs rise --> More capital diverted to extraction
--> Less productive capital --> Lower industrial output --> Less investment
--> Capital depreciates faster than it's replaced --> Output falls further
```

In the model, the fraction of capital allocated to obtaining resources (FCAOR) rises steeply as `fraction_remaining` drops below ~0.5. This is computed in the [Resources](sectors/resources.md) sector and feeds directly into the [Capital](sectors/capital.md) sector's output calculations. The lookup table `fcaor` encodes this non-linear relationship.

**ID:** `resource-collapse`
**Type:** reinforcing
**Chain:** `resources.fraction_remaining` > `capital.industrial_output` > `capital.industrial_capital` > `resources.fraction_remaining`


## 2. Pollution --> Agricultural Decline --> Food Crisis (reinforcing)

**Connects:** [Capital](sectors/capital.md) --> [Pollution](sectors/pollution.md) --> [Agriculture](sectors/agriculture.md)

Industrial activity generates persistent pollution that accumulates over decades. Because pollution assimilation has a long time constant, the damage persists even after industrial output declines.

```
Industrial output generates pollution --> Pollution reduces crop yields
--> Less food per capita --> More investment diverted to agriculture
--> Less investment in industry --> But pollution persists (long assimilation time)
--> Yields continue falling
```

The pollution sector's `pollution_index` feeds into the agriculture sector's land fertility calculation via the `lfdr` (land fertility degradation rate) lookup table. This coupling is evaluated at step 4 (agriculture) using the pollution values from step 5 of the *previous* time step, then updated at step 5 of the current step.

**ID:** `pollution-food`
**Type:** reinforcing
**Chain:** `capital.industrial_output` > `pollution.pollution_index` > `agriculture.land_yield` > `agriculture.food_per_capita`


## 3. Demographic Transition (balancing)

**Connects:** [Capital](sectors/capital.md) --> [Population](sectors/population.md)

This is the one major *negative* (stabilizing) feedback loop. It is why the Technology and Ecotopia scenarios perform better --- they allow this transition to proceed before collapse overwhelms it.

```
Rising income --> Smaller desired family size --> Lower birth rate
--> Slower population growth --> Less resource pressure
```

In the model, industrial output per capita (IOPC) influences desired completed family size (DCFS) through the `dcfs` lookup table, which in turn drives the crude birth rate in the [Population](sectors/population.md) sector. The delay between rising income and falling birth rates is a critical parameter --- the demographic transition takes roughly a generation to complete, and if collapse arrives first, the stabilizing effect never materializes.

**ID:** `demographic-transition`
**Type:** stabilizing
**Chain:** `capital.industrial_output_per_capita` > `population.fertility_rate` > `population.population` > `capital.industrial_output_per_capita`


## 4. Population --> Resource Pressure (reinforcing)

**Connects:** [Population](sectors/population.md) --> [Resources](sectors/resources.md)

Population growth amplifies all demands on the system. More people require more industrial output, which accelerates resource depletion.

```
More people --> More consumption --> Faster resource depletion
--> Economic contraction --> Higher death rates --> Population decline
```

This loop operates through the [Capital](sectors/capital.md) sector: population enters as the denominator in per-capita calculations (IOPC, SOPC, food per capita), and total industrial output scales with population-driven demand. The resource usage rate in the [Resources](sectors/resources.md) sector is a function of industrial output, which itself depends on population.

**ID:** `population-resource`
**Type:** reinforcing
**Chain:** `population.population` > `resources.fraction_remaining` > `capital.industrial_output_per_capita` > `population.life_expectancy`


## 5. Pollution Tipping Point (reinforcing)

**Connects:** [Pollution](sectors/pollution.md) (self-reinforcing)

Once pollution exceeds a threshold, the environment's capacity to absorb it degrades. This creates a self-reinforcing spiral.

```
Pollution rises --> Assimilation time increases --> Pollution accumulates faster
--> Even longer assimilation time --> Runaway pollution
```

The `ppasr` (persistent pollution assimilation rate) lookup table in the [Pollution](sectors/pollution.md) sector encodes this non-linearity. At low pollution indices, assimilation keeps pace with generation. Above a tipping point (roughly `pollution_index > 1.0`), the assimilation rate drops and pollution runs away. The Delay3 structure for pollution appearance adds further lag --- by the time the system "sees" the pollution, it is too late to reverse.

**ID:** `pollution-tipping`
**Type:** reinforcing
**Chain:** `pollution.persistent_pollution` > `pollution.pollution_index` > `pollution.persistent_pollution`


## 6. Food--Population Balance (balancing)

**Connects:** [Agriculture](sectors/agriculture.md) <--> [Population](sectors/population.md)

This classical Malthusian feedback provides some self-correction, but operates too slowly to prevent overshoot.

```
Adequate food --> Lower mortality, higher fertility --> Population grows
--> More mouths to feed --> Lower food per capita --> Higher mortality
```

Food per capita enters the [Population](sectors/population.md) sector through two channels: the life expectancy multiplier from food (`lmf` lookup table), which affects mortality, and the desired family size calculation, which affects fertility. The smoothed food per capita (`food_per_capita_smooth`, a proper ODE stock with a 2-year time constant) prevents the population sector from overreacting to short-term food fluctuations.

**ID:** `food-population`
**Type:** stabilizing
**Chain:** `agriculture.food_per_capita` > `population.life_expectancy` > `population.population` > `agriculture.food_per_capita`
