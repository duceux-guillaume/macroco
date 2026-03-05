# LDCO --- Land Development Cost

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_development_cost`)
**World3-03 name:** (no direct equivalent)
**Status:** Custom / no reference

## Equation Context

$$\text{land\_fraction\_developed} = 1 - \frac{PAL}{3.2 \times 10^9}$$

$$\text{dev\_cost} = \mathrm{LDCO}(\text{land\_fraction\_developed})$$

LDCO maps the fraction of potential arable land already developed to a cost multiplier for developing additional land. Evaluated in the agriculture sector (`crates/world3-core/src/model/sectors/agriculture.rs`).

## Purpose

Maps the fraction of total potential arable land already developed to a cost multiplier for developing additional land. The best and most accessible land is developed first; each successive hectare is more marginal, requiring greater investment in clearing, drainage, irrigation, or soil amendment.

The table follows an approximately exponential growth curve, rising from 100 (when no land is developed) to 616 (when all land is developed). This means the last 10% of potential arable land costs over 6 times more to develop than the first 10%.

$$\text{land\_fraction\_developed} = 1 - \frac{PAL}{3.2 \times 10^9}$$

$$\text{dev\_cost} = \mathrm{LDCO}(\text{land\_fraction\_developed})$$

## Breakpoints

| $$x$$ (fraction developed) | $$y$$ (cost multiplier) |
|---|---|
| 0.0 | 100 |
| 0.1 | 117 |
| 0.2 | 137 |
| 0.3 | 161 |
| 0.4 | 192 |
| 0.5 | 232 |
| 0.6 | 282 |
| 0.7 | 344 |
| 0.8 | 418 |
| 0.9 | 507 |
| 1.0 | 616 |

11 points, uniformly spaced at 0.1 intervals. The growth factor between adjacent points averages approximately 1.19, consistent with an exponential cost curve.

## Relationship to pyworld3

pyworld3 uses DCPH (Development Cost Per Hectare) indexed by PAL (absolute potentially arable land area in hectares). Our parameterization uses the fraction of land already developed as the independent variable, which is dimensionless and invariant to changes in the total land endowment assumption.

## References

- Meadows et al. (2004), land development cost discussion (Chapter 3)
