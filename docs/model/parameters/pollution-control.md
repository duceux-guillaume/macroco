# Pollution Control

Fraction by which pollution generation is reduced per unit of industrial and agricultural output. Represents the aggregate effect of emissions regulations, cleaner production technologies, and end-of-pipe treatment.

**Sector:** [Pollution](../sectors/pollution.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `pollution_control`
**BAU value:** 0.0

| Preset | Value |
|---|---|
| BAU (Collapse) | 0.0 |
| Technology (Technotopia) | 0.8 |
| Stabilized (Ecotopia) | 0.8 |

**Range:** 0.0 -- 1.0 (fraction)
**UI step:** 0.05

## Equation Context

Pollution control enters the generation equation as a simple reduction factor:

$$
G = (G_{\text{ind}} + G_{\text{agr}})(1 - c)
$$

where $c$ is `pollution_control`, clamped to $[0, 1]$. At $c = 0$ (BAU), generation is unabated. At $c = 0.8$ (Technology and Stabilized presets), only 20% of generated pollution enters the appearance pipeline.

This parameter affects both industrial and agricultural pollution generation equally. It does not distinguish between pollution types or control mechanisms -- it is a single aggregate lever representing the overall effectiveness of pollution abatement policy.

## Calibration

In the BAU scenario, pollution control is zero: no policy intervention is assumed. The Technology and Stabilized presets both set $c = 0.8$, representing aggressive but not total pollution abatement. This value was chosen to keep pollution below tipping-point levels in scenarios where industrial output continues to grow, while remaining plausible as a long-term policy target (80% reduction in pollution intensity is ambitious but within the range of historical Clean Air Act achievements for specific pollutants).

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*, Table A-1 -- Scenario parameter settings
- `data/presets/business_as_usual.json`, `data/presets/comprehensive_technology.json`, `data/presets/stabilized_world.json`
