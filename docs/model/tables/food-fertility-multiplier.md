# FRNF --- Food Fertility Multiplier

**Sector:** [Agriculture](../sectors/agriculture.md) / [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`food_fertility_multiplier`)
**World3-03 name:** (no direct equivalent)
**Status:** Custom / no reference

## Equation Context

$$TFR = TFR_{\text{base}} \times \mathrm{FRNF}(\text{food\\_ratio}) \times \ldots$$

The food fertility multiplier (FRNF) modulates total fertility rate based on the food ratio (food per capita / subsistence food per capita). Evaluated in the population sector (`crates/world3-core/src/model/sectors/population.rs`).

## Purpose

Maps the food ratio (food per capita divided by subsistence food per capita) to a multiplier on fertility. Nutritional status affects biological fecundity: severe malnutrition suppresses fertility, adequate nutrition restores it, and surplus food has a modest additional effect.

At zero food, the multiplier is zero (no reproduction possible under starvation). At subsistence (food ratio = 1.0), the multiplier is 1.0 (baseline fertility). Above subsistence, the effect saturates quickly --- a food ratio of 2.0 gives only a 10% boost.

This table creates a weak balancing feedback: declining food slightly reduces birth rates, slowing population growth during food shortages.

## Breakpoints

| $x$ (food ratio) | $y$ (fertility multiplier) |
|---|---|
| 0.0 | 0.0 |
| 0.5 | 0.6 |
| 1.0 | 1.0 |
| 1.5 | 1.05 |
| 2.0 | 1.1 |

5 points. Beyond $$x = 2.0$$, the table clamps at $$y = 1.1$$.

## Relationship to pyworld3

pyworld3 uses FCE (Food Consumption Effect) tables with a different functional role. FCE modulates consumption patterns rather than biological fertility. Our FRNF table provides a direct fertility-nutrition link.

## References

- Meadows et al. (2004), fertility and nutrition discussion (Chapter 2)
