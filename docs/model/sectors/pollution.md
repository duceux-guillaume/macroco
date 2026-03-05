# Pollution Sector

**Source code:** `crates/world3-core/src/model/sectors/pollution.rs`

## Overview

The pollution sector models the accumulation and dissipation of persistent pollution -- long-lived contaminants such as heavy metals, persistent organic pollutants, and greenhouse gases that remain in the environment for decades. Pollution is generated as a byproduct of industrial production and agricultural intensification, then passes through a 20-year appearance delay before entering the persistent stock. The environment assimilates pollution at a rate that degrades sharply as pollution rises, producing a critical tipping-point dynamic: once the pollution index exceeds a threshold, assimilation time lengthens so dramatically that net accumulation accelerates even if generation remains constant.

This sector feeds into the agriculture sector (pollution suppresses land fertility and crop yields) and the population sector (pollution raises mortality), creating two of the model's most consequential cross-sector feedback loops.

## State Variables

The sector maintains four ODE stocks. Three form a Delay3 pipeline that models the 20-year lag between pollution generation and its environmental appearance; the fourth is the persistent pollution stock itself.

| Stock | Field | Units | Initial (1900) |
|---|---|---|---|
| Persistent pollution | `persistent_pollution` | pollution units (1970 $\approx$ 1) | 0.05 |
| Appearance stage 1 | `pollution_appearance_stage1` | pollution units | 0.0167 |
| Appearance stage 2 | `pollution_appearance_stage2` | pollution units | 0.0167 |
| Appearance buffer (stage 3) | `pollution_appearance_buffer` | pollution units | 0.0167 |

The three pipeline stages are initialized to their steady-state values: at 1900 generation $\approx 0.0025$ units/yr and stage time constant $\tau = 20/3 \approx 6.67$ yr, each stage holds $\text{generation} \times \tau \approx 0.0167$.

Two auxiliary fields are also computed each time step:

- `pollution_index` -- equal to $\max(\text{persistent\_pollution},\, 0)$, used as input to the assimilation time lookup and by cross-sector tables (LYMAP, LMP).
- `generation_rate` and `assimilation_rate` -- diagnostic outputs.

## Governing Equations

### Pollution Generation

Industrial and agricultural activity each contribute to pollution generation through separate channels. Both are modulated by a pollution control parameter $c \in [0, 1]$.

$$
\text{iopc\_norm} = \frac{\text{industrial\_output\_per\_capita}}{220}
$$

$$
\text{agri\_norm} = \frac{\text{agricultural\_inputs\_per\_hectare}}{40}
$$

$$
G_{\text{ind}} = \text{industrial\_output} \times \text{PPGIO} \times f_{\text{PPGIO}}(\text{iopc\_norm})
$$

$$
G_{\text{agr}} = \text{arable\_land} \times \text{agri\_inputs\_per\_ha} \times \text{PPGAO} \times f_{\text{PPGAO}}(\text{agri\_norm})
$$

$$
G = (G_{\text{ind}} + G_{\text{agr}})(1 - c)
$$

where the constants are:

- $\text{PPGIO} = 3.0 \times 10^{-13}$ -- industrial pollution coefficient (index units / USD)
- $\text{PPGAO} = 1.0 \times 10^{-13}$ -- agricultural pollution coefficient

At 1970 conditions (industrial output $\approx 10^{12}$ USD/yr, iopc\_norm $\approx 1.0$), industrial generation is approximately 0.30 index units/yr. Agricultural generation at 1970 is roughly 0.005 index units/yr -- two orders of magnitude smaller.

### Delay3 Appearance Pipeline

Generated pollution does not immediately enter the persistent stock. It passes through a three-stage cascaded delay (Delay3) with total delay time PPTD = 20 years, matching the World3-03 specification. Each stage has time constant $\tau = \text{PPTD}/3 \approx 6.67$ years:

$$
\frac{d(\text{stage}_1)}{dt} = G - \frac{\text{stage}_1}{\tau}
$$

$$
\frac{d(\text{stage}_2)}{dt} = \frac{\text{stage}_1}{\tau} - \frac{\text{stage}_2}{\tau}
$$

$$
\frac{d(\text{stage}_3)}{dt} = \frac{\text{stage}_2}{\tau} - \frac{\text{stage}_3}{\tau}
$$

The output of the pipeline is the appearance rate:

$$
A_{\text{appear}} = \frac{\text{stage}_3}{\tau}
$$

The Delay3 structure produces a more uniform transit-time distribution than a simple first-order delay (Delay1), better representing the real-world lag between emission and environmental impact.

### Pollution Assimilation

The environment absorbs persistent pollution at a rate that depends on the current pollution level through a nonlinear lookup table:

$$
T_{\text{assim}} = f_{\text{PPASR}}(\text{pollution\_index})
$$

$$
A_{\text{assim}} = \frac{\text{persistent\_pollution}}{T_{\text{assim}}}
$$

The assimilation time $T_{\text{assim}}$ ranges from 1.5 years at zero pollution to 160 years at pollution index 100. This steep nonlinearity is the mechanism behind the pollution tipping point: once pollution exceeds roughly index 10, assimilation slows so much that the stock accumulates faster than the environment can process it.

### Pollution Index

The pollution index is a direct pass-through of the persistent pollution stock, floored at zero:

$$
\text{pollution\_index} = \max(\text{persistent\_pollution},\, 0)
$$

The stock is pre-normalized so that 1.0 corresponds approximately to the 1970 pollution level (calibrated through the PPGIO/PPGAO coefficients rather than an explicit PPOL70 constant).

### Net Derivative

$$
\frac{d(\text{persistent\_pollution})}{dt} = A_{\text{appear}} - A_{\text{assim}}
$$

## Feedback Loops

### Pollution Tipping Point (Reinforcing -- R)

$$
\text{pollution} \uparrow \;\longrightarrow\; T_{\text{assim}} \uparrow \;\longrightarrow\; A_{\text{assim}} \downarrow \;\longrightarrow\; \text{net accumulation} \uparrow \;\longrightarrow\; \text{pollution} \uparrow
$$

This is the sector's defining reinforcing loop. Once persistent pollution rises above the threshold where assimilation time grows faster than linearly, the system enters a self-amplifying regime. In the BAU scenario, this loop activates around 2020--2040.

### Pollution to Agriculture (Cross-sector -- Balancing)

$$
\text{pollution} \uparrow \;\longrightarrow\; \text{land fertility} \downarrow \;\longrightarrow\; \text{food per capita} \downarrow \;\longrightarrow\; \text{mortality} \uparrow \;\longrightarrow\; \text{population} \downarrow \;\longrightarrow\; \text{industrial output} \downarrow \;\longrightarrow\; \text{pollution generation} \downarrow
$$

Pollution suppresses agriculture through the LYMAP (Land Yield Multiplier from Air Pollution) and LMP (Lifetime Multiplier from Pollution) lookup tables in the agriculture and population sectors respectively. This creates a delayed balancing loop: pollution eventually reduces the industrial activity that generates it, but only after significant damage to food production and life expectancy.

## Deviations from World3-03

### Custom PPGIO and PPGAO Tables

pyworld3 uses constant multipliers for pollution generation from industry and agriculture (i.e., linear scaling). Our implementation replaces these with nonlinear lookup tables that capture diminishing pollution intensity at high output levels. This reflects the empirical observation that pollution per unit GDP tends to decrease as economies mature (environmental Kuznets curve for certain pollutants). See [pollution-generation-industry](../tables/pollution-generation-industry.md) and [pollution-generation-agriculture](../tables/pollution-generation-agriculture.md).

### Structural PPASR Difference

The pollution assimilation mechanism differs structurally from World3-03. pyworld3 uses AHLM (Assimilation Half-Life Multiplier), a dimensionless multiplier applied to a base half-life, with x-axis on the range 1--1001. Our model computes assimilation time directly in years as a function of pollution index (range 0--100). The functional form is completely different, though both achieve the same qualitative behavior: assimilation degrades sharply as pollution rises. See [pollution-assimilation-time](../tables/pollution-assimilation-time.md).

### IOPC Normalization

Our model normalizes IOPC to 220 (approximate 1970 value) rather than 200 as in the model-guide simplified description. This aligns with World3-03's IO70 = $7.9 \times 10^{11}$ USD, yielding IOPC70 $\approx$ \$220/person/yr.

## Lookup Tables

| Table | Variable | x-axis | y-axis | Status |
|---|---|---|---|---|
| [PPGIO](../tables/pollution-generation-industry.md) | `pollution_generation_industry` | IOPC normalized | generation multiplier | Custom |
| [PPGAO](../tables/pollution-generation-agriculture.md) | `pollution_generation_agriculture` | agri inputs normalized | generation multiplier | Custom |
| [PPASR](../tables/pollution-assimilation-time.md) | `pollution_assimilation_time` | pollution index | years | Intentional deviation |

## References

- Meadows, D. H. et al. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green. Chapter 4 (Pollution dynamics).
- Meadows, D. H. et al. (1972). *The Limits to Growth*. Universe Books. Fig. 35 (standard run).
- pyworld3: `functions_table_world3.json` -- PPGIO, PPGAO constants; AHLM table.
- `docs/model/tables/` -- Individual lookup table audit records.
