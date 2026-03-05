# Indicated Service Output Per Capita (ISOPC)

The level of service output per capita that a society demands given its industrial development. As income rises, populations expect more health care, education, and public services, so the indicated service level increases with IOPC.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/lookup/tables.rs`, field `indicated_service_per_capita`
**Status:** Exact match

## Equation Context

ISOPC enters the service allocation fraction as the denominator of the service adequacy ratio:

$$\text{spc\_ratio} = \frac{SOPC}{ISOPC(IOPC)}$$

$$f_{srv} = \text{FIOAS}(\text{spc\_ratio})$$

By making the reference level dynamic rather than a fixed constant, ISOPC ensures that service demand scales with economic development. Without this, a growing economy would quickly satisfy a static service target and stop investing in service capital — an unrealistic outcome.

This table replaces an earlier hardcoded value of $$ISOPC = 200$$ that was used before the March 2026 dynamic ISOPC rework.

## Breakpoints

| IOPC ($/person/yr) | ISOPC ($/person/yr) |
|---|---|
| 0 | 40 |
| 200 | 300 |
| 400 | 640 |
| 600 | 1000 |
| 800 | 1220 |
| 1000 | 1450 |
| 1200 | 1650 |
| 1400 | 1800 |
| 1600 | 2000 |

This table is an exact match to pyworld3's ISOPC1 table, added during the March 2026 dynamic ISOPC rework.

## References

- Meadows et al. (2004), Table ISOPC1 (World3-03 Vensim model)
- pyworld3: `functions_table_world3.json`, key `isopc1`
