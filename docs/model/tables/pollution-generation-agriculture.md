# Pollution Generation from Agriculture (PPGAO)

**Lookup table:** `pollution_generation_agriculture`
**Source code:** `crates/world3-core/src/lookup/tables.rs`
**Sector:** [Pollution](../sectors/pollution.md)
**Status:** Custom / no reference

## Definition

Maps normalized agricultural inputs per hectare to a pollution generation multiplier. The normalization base is 40 USD/ha/yr (approximate 1970 level).

$$
f_{\text{PPGAO}}\!\left(\frac{\text{agri\\_inputs\\_per\\_ha}}{40}\right) \;\longrightarrow\; \text{generation multiplier}
$$

## Breakpoints

| x (inputs / 40) | y (multiplier) |
|---|---|
| 0 | 0.0 |
| 1 | 1.0 |
| 2 | 1.7 |
| 3 | 2.2 |
| 4 | 2.5 |

## Functional Form

The table is concave and monotonically increasing, with the same structural rationale as the industrial generation table. At the normalization point (x = 1), the multiplier equals 1.0 by construction. Doubling agricultural intensity from 1x to 2x raises the multiplier by 0.7 (70%), but doubling from 2x to 4x raises it by only 0.8 (47% relative). This concavity captures the diminishing marginal pollution from agricultural intensification -- initial applications of fertilizer and pesticide have the largest environmental impact per unit, while further intensification yields proportionally less additional pollution.

Beyond x = 4 (inputs $\approx$ 160 USD/ha/yr), the table clamps to the endpoint value of 2.5.

## Deviation from pyworld3

pyworld3 does not use a lookup table for PPGAO. It applies PPGAO as a constant multiplier, making agricultural pollution scale linearly with inputs. Our nonlinear table introduces diminishing marginal pollution, reflecting the empirical pattern that modern high-input agriculture achieves somewhat better pollution-per-output ratios than early-stage intensification.

## Equation Context

Agricultural pollution generation:

$$
G_{\text{agr}} = \text{arable\\_land} \times \text{agri\\_inputs\\_per\\_ha} \times 1.0 \times 10^{-13} \times f_{\text{PPGAO}}\!\left(\frac{\text{agri\\_inputs\\_per\\_ha}}{40}\right)
$$

Agricultural generation is typically two orders of magnitude smaller than industrial generation ($\approx 0.005$ vs. $\approx 0.30$ index units/yr at 1970 conditions), but it becomes significant as industrial output declines in collapse scenarios.

## References

- pyworld3: constant PPGAO coefficient (no table equivalent)
