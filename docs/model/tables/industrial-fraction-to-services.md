# Industrial Fraction to Services (FIOAS)

The fraction of industrial output allocated to building service capital (hospitals, schools, government buildings), as a function of how well current service provision meets demand. When service output per capita is low relative to the indicated level, the economy invests heavily in services; when service demand is satisfied, the allocation drops to zero.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `industrial_fraction_to_services`
**Status:** Exact match

## Equation Context

The service allocation uses a normalized measure of service adequacy:

$$\text{spc\_ratio} = \frac{SOPC}{ISOPC(IOPC)}$$

$$f_{srv} = \text{FIOAS}(\text{spc\_ratio})$$

where $$SOPC$$ is actual service output per capita and [$$ISOPC$$](indicated-service-per-capita.md) is the indicated (desired) level, which itself scales with IOPC. This means that as the economy grows, service demand rises, preventing premature disinvestment in services.

## Breakpoints

| SOPC/ISOPC | Fraction |
|---|---|
| 0 | 0.30 |
| 0.5 | 0.20 |
| 1.0 | 0.10 |
| 1.5 | 0.05 |
| 2.0 | 0.0 |

This table is an exact match to pyworld3's FIOAS1 table.

## References

- Meadows et al. (2004), Table FIOAS1 (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `fioas1`
