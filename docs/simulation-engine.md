# Simulation Engine

The simulation engine lives in `crates/world3-core/` and is a pure Rust library with no I/O. It implements the World 3 system dynamics model from Meadows et al., *Limits to Growth* (1972).

## WorldState

`WorldState` is a typed struct (not `Vec<f64>`) whose fields mirror the published World 3 equations. It contains five sector sub-states:

| Sector | Struct | Key fields |
|--------|--------|------------|
| Population | `PopulationState` | `population`, `cohort_0_14`, `cohort_15_44`, `cohort_45_64`, `cohort_65_plus`, `birth_rate`, `death_rate`, `life_expectancy`, `fertility_rate` |
| Capital | `CapitalState` | `industrial_capital`, `service_capital`, `industrial_output`, `industrial_output_per_capita`, `service_output_per_capita` |
| Agriculture | `AgricultureState` | `arable_land`, `potentially_arable_land`, `food`, `food_per_capita`, `land_yield`, `agricultural_inputs_per_hectare` |
| Resources | `ResourceState` | `nonrenewable_resources`, `fraction_remaining` |
| Pollution | `PollutionState` | `persistent_pollution`, `pollution_index`, `generation_rate`, `assimilation_rate` |

### ODE Stock Variables (10 total)

The solver integrates 10 stock variables. All other fields are auxiliary (derived each step).

| # | Variable | Unit | Sector |
|---|----------|------|--------|
| 1 | `cohort_0_14` | persons | Population |
| 2 | `cohort_15_44` | persons | Population |
| 3 | `cohort_45_64` | persons | Population |
| 4 | `cohort_65_plus` | persons | Population |
| 5 | `industrial_capital` | 1975 USD | Capital |
| 6 | `service_capital` | 1975 USD | Capital |
| 7 | `arable_land` | hectares | Agriculture |
| 8 | `potentially_arable_land` | hectares | Agriculture |
| 9 | `nonrenewable_resources` | dimensionless (0-1) | Resources |
| 10 | `persistent_pollution` | pollution units (1970=1) | Pollution |

`to_vec()` / `from_vec()` convert between the struct and a flat `Vec<f64>` at solver boundaries only.

## Sector Computation Order

The derivative function evaluates sectors in a fixed order to satisfy data dependencies:

1. **Resources** — other sectors need `fraction_remaining` for cost multipliers
2. **Capital** — depends on resource fraction; produces `industrial_output`
3. **Agriculture** — depends on `industrial_output` for inputs and on pollution
4. **Pollution** — depends on `industrial_output` and agricultural inputs
5. **Population** — depends on food, services, and pollution

See `crates/world3-core/src/model/derivatives.rs` for the implementation.

## RK4 Solver

The solver (`crates/world3-core/src/solver/rk4.rs`) uses the classic 4th-order Runge-Kutta method with a fixed time step:

- **Step size**: configurable via `ScenarioParams.time_step` (default 1.0 year)
- **Each RK4 step** computes four derivative evaluations (k1–k4) and combines them with the standard weighted average
- **After each accepted step**, all auxiliary fields are recomputed to ensure consistency
- **Divergence detection**: if population leaves the range [0, 1e13], the solver returns `SolverError::Diverged`
- **Physical bounds**: `from_vec()` clamps all stocks to non-negative values

No adaptive step-size control is used. The fixed-step approach is sufficient for the World 3 dynamics at dt=1.0.

## Lookup Tables

All non-linear relationships in World 3 are encoded as piecewise-linear lookup tables, loaded from `data/lookup_tables/*.json`. The `WorldLookupTables` struct holds all tables and is shared across the solver via `Arc`.

Each `LookupTable` maps an input range to an output range with linear interpolation between breakpoints.

## Presets

Three built-in scenario presets are provided:

| Preset | Key parameters | Description |
|--------|---------------|-------------|
| **BAU** | No interventions (`family_planning_efficacy=0`, `pollution_control=0`, `resource_efficiency=1`) | Original World 3 standard run. Reproduces Meadows 1972 Fig. 35. |
| **Technology** | `resource_efficiency=4`, `pollution_control=0.8`, `agricultural_technology=2`, `technology_growth_rate=0.02` | Aggressive technology gains without social changes. |
| **Stabilized** | Technology params + `family_planning_efficacy=0.95` (from 1975), `land_protection=0.3` | Full combination of technology and social policy. Closest to sustainable. |

See `crates/world3-core/src/model/params.rs` for all parameter definitions and defaults.
