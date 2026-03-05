# Subsistence Food Per Capita

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/model/params.rs` (`subsistence_food_per_capita`)
**Unit:** kg/person/yr (vegetable-equivalent)
**Range:** 150 -- 350
**Step:** 10

## Purpose

The minimum food per capita required for basic health and survival. Below this threshold, life expectancy falls sharply (via the population sector's LMF table) and agricultural investment increases (via the FIOAA allocation fraction). This parameter anchors the food ratio used throughout the model:

$$\text{food\_ratio} = \frac{FPC}{\text{SFPC}}$$

A food ratio of 1.0 means the population is at subsistence. Above 1.0, health improves and agricultural allocation decreases. Below 1.0, mortality rises and the economy shifts resources toward food production.

The value of 230 kg/person/yr corresponds to approximately 1700 kcal/day (assuming ~2700 kcal/kg for a mixed vegetable-equivalent diet), which aligns with the World3-03 specification of SFPC = 230.

## Scenario Values

| Scenario | Value | Rationale |
|----------|-------|-----------|
| BAU (Collapse) | 230.0 | World3-03 standard value |
| Technology (Technotopia) | 230.0 | Same biological minimum |
| Stabilized (Ecotopia) | 230.0 | Same biological minimum |

This parameter is identical across all scenarios because it represents a biological constant (minimum nutritional requirement), not a policy lever. It is exposed as a parameter for sensitivity analysis.

## Equation Context

The subsistence food per capita anchors the food ratio used throughout the model:

$$\text{food\_ratio} = \frac{FPC}{\text{SFPC}}$$

A food ratio of 1.0 means the population is at subsistence. The ratio enters the FIOAA allocation table, the LMF life expectancy table, and the FALM land maintenance table.

## Calibration

The value of 230 kg/person/yr matches World3-03 exactly (SFPC = 230). This represents a biological constant (~1700 kcal/day), not a policy lever, so no calibration deviation is needed. It is identical across all presets.

## Role in the Model

The subsistence food level appears in three places:

1. **FIOAA allocation** (via FALM food ratio): $$\text{food\_ratio}_{\text{FALM}} = FPC_s / \text{SFPC}$$. When food is below subsistence, agricultural allocation increases.

2. **Life expectancy** (via LMF table in population sector): food ratio below 1.0 reduces life expectancy, increasing death rates.

3. **FALM land maintenance**: perceived food ratio ($$FPC_s / \text{SFPC}$$) determines how much output goes to soil maintenance. At subsistence, only 4% goes to maintenance.

## References

- Meadows et al. (2004), SFPC = 230 kg/person/yr
- FAO minimum dietary energy requirement: ~1800 kcal/day (~230 kg vegetable-equivalent/yr)
