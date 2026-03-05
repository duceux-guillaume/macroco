# Fecundity Multiplier (FM)

The biological fecundity multiplier as a function of life expectancy. At low life expectancy, malnutrition and disease reduce the biological capacity for reproduction (FM approaches zero). At moderate LE (60 years), fecundity reaches its natural level (FM = 1.0). At high LE (70--80 years), FM slightly exceeds 1.0, reflecting improved reproductive health in well-nourished populations.

**Sector:** [Population](../sectors/population.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `fecundity_multiplier`
**Status:** Exact match with World3-03

## Equation Context

This table provides $\text{FM}(\text{LE})$ in the biological fertility ceiling:

$$
\text{TFR} = \min\!\bigl(\text{TFR}_{\text{desired}},\; \text{MTF} \times \text{FM}(\text{LE})\bigr)
$$

where $\text{MTF} = 12$ children/woman is the World3-03 maximum total fertility. The biological ceiling ensures that even when desired fertility is very high (e.g., low-income, high-compensation conditions), actual fertility cannot exceed what is physiologically possible given health conditions. See [Population sector](../sectors/population.md).

## Breakpoints

| Life expectancy (years) | FM |
|------------------------:|---:|
| 0 | 0.00 |
| 10 | 0.20 |
| 20 | 0.40 |
| 30 | 0.60 |
| 40 | 0.80 |
| 50 | 0.90 |
| 60 | 1.00 |
| 70 | 1.05 |
| 80 | 1.10 |

Matches pyworld3 exactly. Aligned during March 2026 pyworld3 alignment work. Previously capped at 0.87 --- now allows FM > 1.0 at high LE per World3-03 specification.

## References

- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*. Chelsea Green Publishing.
- pyworld3 reference key: `fm`
