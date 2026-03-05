# End Year

Simulation end year. The solver stops integrating when time reaches this value.

**Sector:** [Solver](../solver.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `end_year`
**BAU value:** 2100.0

## Equation Context

The solver runs from `start_year` to `end_year` in `time_step` increments. The 200-year span (1900–2100) matches the original World3 simulation horizon.

## Calibration

World3-03 standard run: 2100. Extending beyond 2100 is possible but model dynamics become increasingly speculative.

## References

- Meadows et al. 2004 — Standard 1900–2100 horizon
