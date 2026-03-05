# Pollution Generation from Industry (PPGIO)

**Lookup table:** `pollution_generation_industry`
**Source code:** `crates/world3-core/src/lookup/tables.rs`
**Sector:** [Pollution](../sectors/pollution.md)
**Status:** Custom / no reference

## Definition

Maps normalized industrial output per capita to a pollution generation multiplier. The normalization base is 220 USD/person/yr (approximate 1970 IOPC).

$$
f_{\text{PPGIO}}\!\left(\frac{\text{IOPC}}{220}\right) \;\longrightarrow\; \text{generation multiplier}
$$

## Breakpoints

| x (IOPC / 220) | y (multiplier) |
|---|---|
| 0 | 0.0 |
| 1 | 1.0 |
| 2 | 1.5 |
| 3 | 1.9 |
| 4 | 2.16 |
| 5 | 2.36 |

## Functional Form

The table is concave and monotonically increasing. At the normalization point (x = 1), the multiplier equals 1.0 by construction. As IOPC grows beyond 1970 levels, pollution generation continues to rise but with diminishing marginal intensity -- doubling IOPC from 1x to 2x increases the multiplier by 0.5 (50%), while doubling from 2x to 4x increases it by only 0.66 (44%).

Beyond x = 5 (IOPC $\approx$ \$1,100/person/yr), the table clamps to the endpoint value of 2.36. This saturation reflects the empirical pattern that very high-income economies shift toward cleaner production technologies.

## Deviation from pyworld3

pyworld3 does not use a lookup table for PPGIO. Instead, it applies PPGIO as a constant multiplier (pollution generation scales linearly with industrial output). Our nonlinear table captures diminishing pollution intensity at high output levels, consistent with observed environmental Kuznets curve dynamics for industrial pollutants.

## Equation Context

Industrial pollution generation:

$$
G_{\text{ind}} = \text{IO} \times 3.0 \times 10^{-13} \times f_{\text{PPGIO}}\!\left(\frac{\text{IOPC}}{220}\right)
$$

At 1970 (IO $\approx 10^{12}$, IOPC/220 $\approx 1.0$): $G_{\text{ind}} \approx 10^{12} \times 3 \times 10^{-13} \times 1.0 = 0.30$ index units/yr.

## References

- pyworld3: constant PPGIO coefficient (no table equivalent)
