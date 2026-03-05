# Pollution Assimilation Time (PPASR)

**Lookup table:** `pollution_assimilation_time`
**Source code:** `crates/world3-core/src/lookup/tables.rs`
**Sector:** [Pollution](../sectors/pollution.md)
**Status:** Intentional deviation

## Definition

Maps the pollution index to the time required for the environment to assimilate persistent pollution, in years. Higher pollution levels overwhelm natural assimilation capacity, causing the time to grow steeply.

$$
T_{\text{assim}} = f_{\text{PPASR}}(\text{pollution\_index}) \quad [\text{years}]
$$

## Breakpoints

| x (pollution index) | y (assimilation time, years) |
|---|---|
| 0 | 1.5 |
| 1 | 2.5 |
| 2.5 | 5.0 |
| 5 | 10.0 |
| 10 | 20.0 |
| 25 | 40.0 |
| 50 | 80.0 |
| 100 | 160.0 |

## Functional Form

The table is monotonically increasing and approximately log-linear: doubling the pollution index roughly doubles the assimilation time across most of the range. At low pollution (index < 1, pre-industrial conditions), the environment assimilates pollution in 1.5--2.5 years. At the 1970 reference level (index $\approx$ 1), assimilation takes about 2.5 years. In Collapse scenarios where the pollution index reaches 25--50, assimilation times of 40--80 years effectively render the pollution permanent on human timescales.

This steep nonlinearity creates the pollution tipping point: once the pollution index exceeds roughly 10, the assimilation time grows faster than the pollution stock, leading to runaway accumulation. The table is calibrated so that Collapse produces visible pollution buildup (index > 5) by 2000 and peaks above 10 by 2030--2040.

Beyond x = 100, the table clamps to 160 years.

## Deviation from pyworld3

This table is structurally different from the pyworld3 equivalent. pyworld3 uses AHLM (Assimilation Half-Life Multiplier), a dimensionless multiplier applied to a base assimilation half-life:

| | x-axis | y-axis | Mechanism |
|---|---|---|---|
| **pyworld3 (AHLM)** | 1, 251, 501, 751, 1001 | 1, 11, 21, 31, 41 | dimensionless multiplier on base half-life |
| **Ours (PPASR)** | 0, 1, 2.5, 5, 10, 25, 50, 100 | 1.5, 2.5, 5.0, 10.0, 20.0, 40.0, 80.0, 160.0 | assimilation time directly in years |

The differences are:

1. **x-axis range.** pyworld3 operates on a pollution scale of 1--1001; ours uses pollution index 0--100. The scales are not directly comparable because our pollution stock is normalized differently (1.0 $\approx$ 1970 level via PPGIO/PPGAO coefficients).

2. **y-axis semantics.** pyworld3 AHLM is dimensionless and multiplied by a base half-life to obtain the actual half-life. Our table directly outputs the assimilation time in years, eliminating the intermediate step.

3. **Assimilation computation.** pyworld3 converts pollution via a half-life model; our model uses direct division: $A = \text{PP} / T_{\text{assim}}$. Both achieve the same qualitative dynamics (assimilation degrades sharply with pollution), but the quantitative calibration is independent.

## Equation Context

$$
A_{\text{assim}} = \frac{\text{persistent\_pollution}}{f_{\text{PPASR}}(\text{pollution\_index})}
$$

This is the only sink term for persistent pollution. When $T_{\text{assim}}$ is small (low pollution), the environment cleans up quickly. When $T_{\text{assim}}$ is large (high pollution), assimilation effectively stalls and pollution accumulates without bound until generation declines (typically through industrial collapse).

## References

- pyworld3: `functions_table_world3.json`, AHLM table
- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*, Chapter 4 -- Pollution assimilation dynamics
