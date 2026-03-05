# Pollution Control

Fraction by which pollution generation is reduced per unit of industrial and agricultural output. Represents the aggregate effect of emissions regulations, cleaner production technologies, and end-of-pipe treatment.

**Sector:** [Pollution](../sectors/pollution.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `pollution_control`
**BAU value:** 0.0

| Preset | Value |
|---|---|
| Collapse | 0.0 |
| Technotopia | 0.15 |
| Ecotopia | 0.8 |

**Range:** 0.0 -- 1.0 (fraction)
**UI step:** 0.05

## Equation Context

Pollution control enters the generation equation as a simple reduction factor:

$$
G = (G_{\text{ind}} + G_{\text{agr}})(1 - c)
$$

where $c$ is `pollution_control`, clamped to $[0, 1]$. At $c = 0$ (Collapse), generation is unabated. At $c = 0.8$ (Technology and Ecotopia presets), only 20% of generated pollution enters the appearance pipeline.

This parameter affects both industrial and agricultural pollution generation equally. It does not distinguish between pollution types or control mechanisms -- it is a single aggregate lever representing the overall effectiveness of pollution abatement policy.

## Calibration

In the Collapse scenario, pollution control is zero: no policy intervention is assumed. The Technology and Ecotopia presets both set $c = 0.8$, representing aggressive but not total pollution abatement. This value was chosen to keep pollution below tipping-point levels in scenarios where industrial output continues to grow, while remaining plausible as a long-term policy target (80% reduction in pollution intensity is ambitious but within the range of historical Clean Air Act achievements for specific pollutants).

## Info Panel

**Unit:** fraction (0--1)

**Beginner:** How much pollution is prevented at the source. 0 = no control, 0.8 = 80% of pollution eliminated before it enters the environment.

**Expert:** generation = (gen_industry + gen_agriculture) × (1 -- pollution_control). Clamped to [0, 1].

**Feedback loops:** pollution-food, pollution-tipping

**Related variables:** pollution.persistent_pollution, pollution.pollution_index, agriculture.food_per_capita

**Impact increase:** Less pollution -- protects food production and avoids pollution tipping point

**Impact decrease:** More pollution accumulates -- food yields drop, pollution may spiral

**Sparkline variable:** pollution.pollution_index

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*, Table A-1 -- Scenario parameter settings
- `data/presets/collapse.json`, `data/presets/technotopia.json`, `data/presets/ecotopia.json`
