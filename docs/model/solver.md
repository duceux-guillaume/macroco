# Solver

## Overview

The simulation is a set of 21 quantities (population cohorts, delay pipeline stages, capital stocks, arable land, resources, pollution) that change over time. At each time step, the model calculates how fast each quantity is changing, then advances them forward.

Think of it like an accountant updating a ledger: at the end of each "year," you check the births, deaths, investment, depreciation, extraction, and pollution, then update all the balances.


## Integration Method

The model uses a **4th-order Runge-Kutta (RK4)** solver. RK4 is a standard numerical method that evaluates the derivatives at four points within each time step to get an accurate estimate of the change. For a 200-year simulation at dt = 1.0 year, this means 4 derivative evaluations per step, or roughly 800 total.

The 21 ODE stock variables are packed into a vector via `WorldState::to_vec()`, the RK4 step is computed, and the result is unpacked via `WorldState::from_vec()`. This avoids manual derivative arithmetic on the typed struct.

The classic RK4 formula is:

```
k1 = f(t,       y)
k2 = f(t + dt/2, y + k1 * dt/2)
k3 = f(t + dt/2, y + k2 * dt/2)
k4 = f(t + dt,   y + k3 * dt)

y_{n+1} = y_n + (dt / 6) * (k1 + 2*k2 + 2*k3 + k4)
```

After each step, auxiliary fields (food per capita, industrial output, pollution index, etc.) are recomputed on the accepted state so the stored trajectory has fully populated values for output and visualization.

The solver includes a divergence check after each step: if population, industrial capital, or persistent pollution becomes non-finite or exceeds physical bounds, the simulation halts with a `SolverError::Diverged` error rather than producing garbage output.

**Note on `from_vec()`:** This function zeroes all auxiliary fields --- only the 21 ODE stocks survive. This means that inter-sector feedback in intermediate RK4 stages (k2, k3, k4) relies on re-derivation from stock values, not cached auxiliaries. For feedback that must be consistent across solver stages, the model uses ODE stocks (e.g., `food_per_capita_smooth`) rather than auxiliaries.

**Source code:** `crates/world3-core/src/solver/rk4.rs`


## Sector Evaluation Order

Sector evaluation order matters --- each sector uses the latest values from previously computed sectors within the same step. The order is fixed to satisfy data dependencies:

1. **Pre-seed food per capita** --- If the `food_per_capita` auxiliary is zero (as happens after `from_vec()` in intermediate RK4 stages), estimate it from arable land, land fertility, and population so downstream sectors get a reasonable value.

2. **[Resources](sectors/resources.md) --- auxiliary calculations** --- Computes `fraction_remaining` and the cost multiplier (FCAOR). Capital needs these to determine how much output is diverted to resource extraction.

3. **[Capital](sectors/capital.md) --- industrial and service output** --- Produces `industrial_output` and `industrial_output_per_capita`, which agriculture, pollution, and population all depend on.

4. **[Resources](sectors/resources.md) --- depletion rate** --- Computes the actual resource depletion derivative. Runs after capital because the usage rate depends on industrial output per capita (IOPC).

5. **[Agriculture](sectors/agriculture.md) --- food production** --- Computes food per capita and land dynamics. Depends on industrial output (for agricultural inputs) and on pollution (for land fertility degradation).

6. **[Pollution](sectors/pollution.md) --- generation and assimilation** --- Computes pollution generation from industrial and agricultural activity, and the Delay3 pipeline for pollution appearance. Must run before population so that `pollution_index` is current.

7. **[Population](sectors/population.md) --- births, deaths, aging** --- The final sector to evaluate, because it depends on food (from agriculture), services (from capital), and pollution.

**Source code:** `crates/world3-core/src/model/derivatives.rs`


## Lookup Tables

Non-linear relationships (like "how does pollution affect crop yields?") are encoded as **piecewise-linear lookup tables** --- a series of (x, y) breakpoints with linear interpolation between them. There are 34 tables in total, loaded at startup from `crates/world3-core/src/lookup/tables.rs`.

Lookup tables come directly from the published World 3 model documentation (Meadows et al. 2004). They encode empirical relationships that are not easily expressed as simple equations.

`LookupTable::eval()` clamps to endpoint y-values beyond the x-range (no extrapolation). When adding scenario parameters that push inputs beyond existing table ranges, the table must be extended.

See individual table documentation in [tables/](tables/).


## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| Start Year | 1900.0 | Simulation start |
| End Year | 2100.0 | Simulation end |
| Time Step | 1.0 yr | Integration step size (RK4) |

The original World 3 model (Meadows et al. 2004) used a 0.5-year Euler step. Our RK4 solver achieves comparable or better accuracy with a 1.0-year step, since RK4 has 4th-order error scaling (error ~ dt^4) compared to Euler's 1st-order (error ~ dt). The `diagnose --stability-check` CLI command verifies convergence by running at dt, dt/2, and dt/4 and comparing results.


## References

- Press, W. H., Teukolsky, S. A., Vetterling, W. T., & Flannery, B. P. (2007). *Numerical Recipes: The Art of Scientific Computing* (3rd ed.), Ch. 17. Cambridge University Press.
- Meadows, D. H., Randers, J., & Meadows, D. L. (2004). *Limits to Growth: The 30-Year Update*, Appendix (TIME STEP = 0.5, Euler method). Chelsea Green.
