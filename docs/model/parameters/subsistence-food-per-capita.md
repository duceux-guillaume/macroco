# Subsistence Food Per Capita

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`subsistence_food_per_capita`)
**BAU value:** `230.0` kg/person/yr (matches World3-03 SFPC = 230; ~1700 kcal/day)

## Equation Context

The subsistence food per capita anchors the food ratio used throughout the model:

$$\text{food\\_ratio} = \frac{FPC}{\text{SFPC}}$$

A food ratio of 1.0 means the population is at subsistence. The ratio enters the FIOAA allocation table, the LMF life expectancy table, and the FALM land maintenance table.

## Calibration

The value of 230 kg/person/yr matches World3-03 exactly (SFPC = 230). This represents a biological constant (~1700 kcal/day), not a policy lever, so no calibration deviation is needed. It is identical across all presets.

## Role in the Model

The subsistence food level appears in three places:

1. **FIOAA allocation** (via FALM food ratio): $$\text{food\\_ratio}_{\text{FALM}} = FPC_s / \text{SFPC}$$. When food is below subsistence, agricultural allocation increases.

2. **Life expectancy** (via LMF table in population sector): food ratio below 1.0 reduces life expectancy, increasing death rates.

3. **FALM land maintenance**: perceived food ratio ($$FPC_s / \text{SFPC}$$) determines how much output goes to soil maintenance. At subsistence, only 4% goes to maintenance.

## Info Panel

**Unit:** kg/person/yr

**Beginner:** The minimum food per person needed for basic health. Below this level, life expectancy drops sharply.

**Expert:** Denominator in food_ratio = FPC / subsistence_food. Drives multiple lookup tables. Default 230 kg/yr.

**Feedback loops:** food-population

**Related variables:** agriculture.food_per_capita, population.life_expectancy, population.death_rate

**Impact increase:** Higher bar for adequate nutrition -- more people classified as food-insecure

**Impact decrease:** Lower nutrition threshold -- fewer people in food crisis at same production

**Sparkline variable:** agriculture.food_per_capita

## References

- Meadows et al. (2004), SFPC = 230 kg/person/yr
- FAO minimum dietary energy requirement: ~1800 kcal/day (~230 kg vegetable-equivalent/yr)
