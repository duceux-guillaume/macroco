# LFDR --- Land Fertility Degradation Rate

**Sector:** [Agriculture](../sectors/agriculture.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs` (`land_fertility_degradation`)
**World3-03 name:** LFDR1t
**Status:** Exact match with pyworld3

## Purpose

Maps the persistent pollution index to an annual degradation rate for land fertility. Pollution contaminates soil through heavy metals, acid deposition, and chemical residues, reducing the land's inherent productivity.

At low pollution (index < 10), degradation is negligible. As pollution rises, degradation accelerates sharply: at pollution index 20, half a percent of fertility is lost per year; at index 30, the rate reaches 0.5/yr (halving fertility in under two years).

This table is the primary pathway through which pollution destroys agricultural capacity in the collapse phase. Combined with the erosion multiplier (LERD), it creates a reinforcing loop: pollution degrades fertility, reducing food, which triggers more industrial allocation, generating more pollution.

## Breakpoints

| $x$ (pollution index) | $y$ (degradation rate, yr$^{-1}$) |
|---|---|
| 0 | 0.0 |
| 10 | 0.1 |
| 20 | 0.3 |
| 30 | 0.5 |

4 points. Beyond $$x = 30$$, the table clamps at $$y = 0.5$$.

## Equation Context

$$\text{degradation} = LFERT \times \mathrm{LFDR}(\text{pollution\\_index})$$

$$\frac{dLFERT}{dt} = \text{regeneration} - \text{degradation}$$

## Audit Notes

Aligned to pyworld3 during March 2026 pyworld3 alignment work. Previously had 60% lower degradation rates with an extended x-range.

## References

- Meadows et al. (2004), Table LFDR1t
- pyworld3: `functions_table_world3.json`, key `LFDR1t`
