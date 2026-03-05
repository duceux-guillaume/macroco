# Capital Fraction for Resource Extraction (FCAOR)

**World3-03 name:** FCAOR1 (Fraction of Capital Allocated to Obtaining Resources, policy 1)

**Source code:** `crates/world3-core/src/lookup/tables.rs`

**Status:** Exact match

## Equation Context

$$IO = \frac{IC \times (1 - \text{clamp}(\text{FCAOR}(f_r), 0, 0.95)) \times T}{ICOR}$$

FCAOR maps the fraction of non-renewable resources remaining ($$f_r$$) to the fraction of industrial capital diverted to resource extraction. Evaluated in the capital sector (`crates/world3-core/src/model/sectors/capital.rs`).

## Purpose

This table governs the key feedback loop that drives BAU collapse. As non-renewable resources deplete, an increasing fraction of industrial capital must be diverted from productive output to resource extraction — mining deeper deposits, processing lower-grade ores, exploring more remote sites. The table maps the fraction of resources remaining ($$f_r$$) to the fraction of total industrial capital consumed by extraction activities.

When resources are abundant ($$f_r > 0.6$$), only 5% of capital is needed for extraction. As resources fall below 50%, the extraction fraction rises sharply, and below 10% remaining it approaches 90%. This nonlinear relationship is what makes resource depletion catastrophic rather than gradual: the economy must spend ever more just to maintain access to a shrinking resource base.

## Breakpoints

| $$f_r$$ (fraction remaining) | FCAOR (fraction of capital to extraction) |
|---|---|
| 0.0 | 1.00 |
| 0.1 | 0.90 |
| 0.2 | 0.70 |
| 0.3 | 0.50 |
| 0.4 | 0.20 |
| 0.5 | 0.10 |
| 0.6 | 0.05 |
| 0.7 | 0.05 |
| 0.8 | 0.05 |
| 0.9 | 0.05 |
| 1.0 | 0.05 |

Input domain: $$f_r \in [0, 1]$$. Output range: $$[0.05, 1.0]$$. Interpolation is piecewise-linear between breakpoints. Beyond the domain, `LookupTable::eval()` clamps to endpoint values.

## Interpretation

The table has three distinct regimes:

1. **Abundance** ($$f_r \geq 0.6$$): FCAOR = 0.05. Resources are cheap to extract; only 5% of industrial capital is needed. The economy operates at near-full productive capacity.

2. **Transition** ($$0.2 \leq f_r < 0.6$$): FCAOR rises from 0.05 to 0.70. Resource costs escalate rapidly. Each percentage point of depletion diverts noticeably more capital. This is the zone where BAU enters its crisis phase — industrial output peaks and begins to fall.

3. **Exhaustion** ($$f_r < 0.2$$): FCAOR exceeds 0.70. The economy is trapped in an extraction spiral — most capital goes to maintaining resource flows, leaving little for productive output, investment, or services. Collapse becomes self-reinforcing.

## Usage in the Model

The table is evaluated in the capital sector (`crates/world3-core/src/model/sectors/capital.rs`), not in the resources sector itself:

$$IO = \frac{IC \times (1 - \text{clamp}(\text{FCAOR}(f_r), 0, 0.95)) \times T}{ICOR}$$

where $$IC$$ is industrial capital, $$T$$ is the technology multiplier, and $$ICOR$$ is the incremental capital-output ratio. The FCAOR fraction is clamped to a maximum of 0.95 to prevent complete economic shutdown (a numerical safeguard; the table itself never exceeds 1.0).

## Comparison with pyworld3

| | x breakpoints | y breakpoints |
|---|---|---|
| **Ours** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.7, 0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.05 |
| **pyworld3 (FCAOR1)** | 0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 | 1.0, 0.9, 0.7, 0.5, 0.2, 0.1, 0.05, 0.05, 0.05, 0.05, 0.05 |

The breakpoints are identical. Aligned to pyworld3 during March 2026 audit work. A previous version used a smoother, higher-allocation curve with a linear decline to zero; this was replaced with the exact World3-03 values.

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*, Table A-23 (FCAOR1).
- pyworld3: `functions_table_world3.json`, key `"fcaor1"`.
