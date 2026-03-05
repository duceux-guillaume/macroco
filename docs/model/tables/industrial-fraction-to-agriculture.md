# Industrial Fraction to Agriculture (FIOAA)

The fraction of industrial output allocated to agricultural investment, as a function of the food ratio. When food is scarce relative to the indicated level, the economy diverts more output to agriculture; when food is abundant, agricultural investment falls.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `industrial_fraction_to_agriculture`
**Status:** Intentional deviation

## Equation Context

The food ratio is computed from smoothed food per capita and the [indicated food per capita](indicated-food-per-capita.md):

$$\text{food\_ratio} = \frac{FPC_{smooth}}{IFPC(IOPC)}$$

$$f_{agr} = \text{FIOAA}(\text{food\_ratio})$$

The smoothed FPC is an ODE stock (preserved across RK4 solver stages), while IFPC scales with industrialization to prevent zero-allocation traps at high food levels. At low IOPC (Collapse scenario), IFPC approximates subsistence food (230 kg/person/yr), so the allocation behaves like the original World3-03 formulation.

## Breakpoints

| food_ratio | Macroco | World3-03 | Δ |
|---|---|---|---|
| 0 | 0.40 | 0.40 | 0.00 |
| 0.5 | 0.22 | 0.20 | +0.02 |
| 1.0 | 0.12 | 0.10 | +0.02 |
| 1.5 | 0.04 | 0.025 | +0.015 |
| 2.0 | 0.01 | 0.0 | +0.01 |
| 2.5 | 0.005 | 0.0 | +0.005 |
| 4.0 | 0.005 | — | extended |

## Deviation Rationale

Three deliberate changes:

1. **Floor of 0.005.** The pyworld3 table drops to zero at food_ratio >= 2.0, meaning no industrial output goes to agriculture when food is abundant. In the Technology and Stabilized presets, this caused oscillation: agricultural investment drops to zero, yields decline, food falls, investment spikes, and the cycle repeats. The 0.005 floor maintains a minimal baseline investment that prevents this oscillatory mode.

2. **Slightly higher values at moderate food ratio.** At food_ratio = 0.5, our value is 0.22 versus 0.20; at 1.0, it is 0.12 versus 0.10. The higher allocation compensates for the Land Fraction Harvested (LFH = 0.7) and Processing Loss (PL = 0.1) factors that reduce effective food output by a factor of 0.63 compared to raw agricultural production.

3. **Extended x-range to 4.0.** The pyworld3 table stops at food_ratio = 2.5. In scenarios with high agricultural technology, food ratio can exceed this range. The extension ensures well-defined behavior at extreme values.

**Impact:** Collapse trajectory is nearly unchanged (food ratio stays below 2.0). Technology and Stabilized presets become stable without oscillation.

## References

- Meadows et al. (2004), Table FIOAA1 (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `fioaa1`
