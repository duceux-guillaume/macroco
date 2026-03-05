# Consumption Fraction (FIOACV)

The fraction of industrial output allocated to household consumption goods, as a function of industrial output per capita. As societies grow wealthier, a larger share of output goes to consumer goods rather than investment or services.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `consumption_fraction`
**Status:** Intentional deviation

## Equation Context

The consumption fraction enters the capital allocation system:

$$f_{con} = \text{FIOACV}(IOPC)$$

$$f_{inv} = \max\!\Big(1 - f_{con} - f_{srv} - f_{agr},\; 0\Big)$$

Investment is the residual after consumption, services, and agriculture are satisfied. A higher consumption fraction directly reduces capital formation.

## Breakpoints

| IOPC ($/person/yr) | Macroco | World3-03 | Δ |
|---|---|---|---|
| 0 | 0.30 | 0.30 | 0.00 |
| 80 | 0.32 | 0.32 | 0.00 |
| 160 | 0.34 | 0.34 | 0.00 |
| 240 | 0.36 | 0.36 | 0.00 |
| 320 | 0.38 | 0.38 | 0.00 |
| 400 | 0.40 | 0.43 | −0.03 |
| 480 | 0.44 | 0.73 | −0.29 |
| 560 | 0.49 | 0.77 | −0.28 |
| 640 | 0.55 | 0.81 | −0.26 |
| 720 | 0.62 | 0.82 | −0.20 |
| 800 | 0.70 | 0.83 | −0.13 |

Note: pyworld3 uses IOPC/IOPCD ratio (IOPCD ~400) as x-axis. The World3-03 column above maps pyworld3 x-values 0, 0.2, ..., 2.0 to absolute IOPC 0, 80, ..., 800 for comparison.

## Deviation Rationale

Three deliberate changes from the World3-03 specification:

1. **Different x-axis.** Our table uses absolute IOPC (1975 USD/person/yr) rather than the IOPC/IOPCD ratio. This simplifies the implementation since IOPCD (desired IOPC) would require an additional state variable.

2. **Cap at 0.70.** The pyworld3 table reaches 0.83 at high income, meaning 83% of industrial output goes to consumption. Real-world household consumption is approximately 55-60% of GDP. The 0.70 cap is more empirically grounded and prevents the economy from starving investment at moderate income levels.

3. **Smoothed above IOPC = 400.** The pyworld3 table has a sharp discontinuity at IOPC/IOPCD = 1.0, where consumption jumps from 0.43 to 0.73. This creates an "IOPC trap" — once IOPC reaches ~400, most additional output is consumed, preventing further capital accumulation. Our smoothed curve eliminates this trap while preserving the qualitative shape (rising consumption share with income).

**Impact:** Moderate. Allows IOPC to continue growing through the mid-income range, producing a historical calibration RMSE below 19% for IOPC (REQ-026). The Collapse trajectory is preserved because resource depletion, not consumption, is the binding constraint.

## References

- Meadows et al. (2004), Table FIOAC1 (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `fioac1`
