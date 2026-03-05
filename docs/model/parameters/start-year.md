# Start Year

Simulation start year. All state variables are initialized to their 1900 values at this point.

**Sector:** [Solver](../solver.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `start_year`
**BAU value:** 1900.0

## Equation Context

The solver begins integrating at `start_year` and advances in `time_step` increments until `end_year`.

## Calibration

World3-03 standard run: 1900. All historical calibration assumes a 1900 start. Initial conditions for population cohorts, capital stocks, and resources are set to match 1900 estimates from Meadows et al.

## References

- Meadows et al. 2004, Appendix A — Initial conditions
