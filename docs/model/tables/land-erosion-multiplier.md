# LERD --- Land Erosion Multiplier

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_erosion_multiplier`)
**World3-03 name:** (no direct equivalent)
**Status:** Custom / no reference

## Equation Context

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\\_ratio}) \times (1 - \text{land\\_protection\\_fraction})$$

LERD maps the yield ratio (actual yield / inherent fertility of 600 kg/ha/yr) to an erosion rate multiplier. Evaluated in the agriculture sector (`crates/world3-core/src/model/sectors/agriculture.rs`).

## Purpose

Maps the land yield ratio (actual yield divided by inherent land fertility of 600 kg/ha/yr) to an erosion multiplier. Intensive farming that pushes yields above the inherent level accelerates soil degradation. At a yield ratio of 1.0 (yield equals inherent fertility), the multiplier is 0.7. Above 1.0, erosion rises steeply --- at twice inherent yield, the erosion rate is 2.5 times the baseline.

This captures the empirical observation that high-input industrial agriculture degrades soil faster than low-input traditional farming. The multiplier enters the erosion equation:

$$\text{erosion} = AL \times 0.001 \times \mathrm{LERD}(\text{yield\\_ratio}) \times (1 - \text{land\\_protection\\_fraction})$$

## Breakpoints

| $x$ (yield ratio) | $y$ (erosion multiplier) |
|---|---|
| 0.00 | 0.0 |
| 0.25 | 0.1 |
| 0.50 | 0.3 |
| 0.75 | 0.5 |
| 1.00 | 0.7 |
| 1.25 | 1.0 |
| 1.50 | 1.5 |
| 1.75 | 2.0 |
| 2.00 | 2.5 |

9 points, uniformly spaced at 0.25 intervals.

## Relationship to pyworld3

pyworld3 uses LLMY (Land Life Multiplier from Yield) tables which serve a related but structurally different role. LLMY modulates land lifetime rather than erosion rate directly. Our LERD table provides a more direct erosion-rate multiplier parameterization.

## References

- Meadows et al. (2004), land erosion dynamics discussion (Chapter 3)
