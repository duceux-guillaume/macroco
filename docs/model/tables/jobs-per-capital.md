# Jobs Per Industrial Capital Unit (JPICU)

The number of jobs created per unit of industrial capital, as a function of industrial output per capita. As economies industrialize, production becomes more capital-intensive and automated, so each unit of capital supports fewer workers.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `jobs_per_capital`
**Status:** Exact match

## Equation Context

JPICU is used in employment calculations:

$$\text{jobs} = IC \cdot \text{JPICU}(IOPC)$$

At low IOPC ($$\$50$$/person/yr), each unit of capital supports 0.00037 jobs — labor-intensive production typical of early industrialization. At high IOPC ($$\$800$$/person/yr), this drops to 0.00006 jobs per unit — highly automated, capital-intensive industry.

## Breakpoints

| IOPC ($/person/yr) | Jobs/capital |
|---|---|
| 50 | 0.00037 |
| 200 | 0.00018 |
| 350 | 0.00012 |
| 500 | 0.00009 |
| 650 | 0.00007 |
| 800 | 0.00006 |

This table is an exact match to pyworld3's JPICU table. Aligned during the March 2026 pyworld3 alignment work (previously had an inverted relationship).

## References

- Meadows et al. (2004), Table JPICU (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `jpicu`
