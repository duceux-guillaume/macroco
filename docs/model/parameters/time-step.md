# Time Step

Integration time step for the RK4 solver. Controls the granularity of the numerical simulation.

**Sector:** [Solver](../solver.md)
**Source code:** `crates/world3-core/src/model/params.rs`, field `time_step`
**BAU value:** 1.0 year

## Equation Context

The RK4 solver advances all 21 ODE stocks by `time_step` at each iteration. Within each step, the solver evaluates derivatives at four sub-points (the Runge-Kutta stages) to achieve fourth-order accuracy.

## Calibration

Default of 1.0 year provides stable results for all presets (Collapse, Technology, Stabilized). dt-sensitivity testing (`diagnose --stability-check`) confirms all variables converge within 3% between dt=1.0 and dt=0.25. Pollution peak is the most dt-sensitive variable (~2.4% drift).

World3-03 Vensim uses TIME STEP = 0.5 with Euler integration. Our RK4 solver allows larger steps for equivalent accuracy.

## References

- Meadows et al. 2004 — TIME STEP = 0.5 (Euler)
- Press et al. 2007, *Numerical Recipes*, Ch. 17 — RK4 method
