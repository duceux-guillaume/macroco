# Labor Force Participation (LFP)

The fraction of the total population that participates in the labor force, as a function of demographic age structure. Societies with a larger working-age share have higher labor force participation.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `labor_force_participation`
**Status:** Custom / no reference

## Equation Context

LFP maps the fraction of population aged 15-64 to an aggregate labor force participation rate:

$$LFP = \text{LFP\\_table}(\text{frac\\_working\\_age})$$

This table is used for display and employment diagnostics rather than as a driver of core model dynamics.

## Breakpoints

| Fraction age 15-64 | Labor force fraction |
|---|---|
| 0.50 | 0.50 |
| 0.60 | 0.55 |
| 0.70 | 0.60 |
| 0.80 | 0.65 |

## Deviation Rationale

There is no direct equivalent in pyworld3 or the published World3-03 documentation. This is a custom table created for the Macroco implementation to support employment-related display variables. The linear relationship (roughly 0.5 participation at 50% working-age, rising to 0.65 at 80% working-age) is a simplified approximation of ILO labor force statistics.

## References

- ILO labour force participation estimates (custom derivation)
