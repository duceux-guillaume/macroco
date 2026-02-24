# macroco

Online live macroeconomic model based on the World 3 system dynamics model (Meadows et al., *Limits to Growth*). Extended with modern indicators: climate, energy mix, biodiversity, inequality.

**Status: Phase 1 of 6 complete** — core simulation engine + CLI

## Prerequisites

- Rust 1.75+
- cargo

## Build

```bash
cargo build --workspace
```

## CLI Usage

All commands use the `world3-cli` binary:

```bash
cargo run --bin world3-cli -- <SUBCOMMAND> [OPTIONS]
```

### `simulate`

Run a simulation and output results.

```bash
cargo run --bin world3-cli -- simulate [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `bau` | Scenario preset: `bau`, `technology`, `stabilized` |
| `--output <FILE>` | _(none)_ | Write results to CSV file; prints summary table to stdout if omitted |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |

**Examples:**

```bash
# Print summary table to stdout (BAU, 1900–2100)
cargo run --bin world3-cli -- simulate

# Write full CSV output
cargo run --bin world3-cli -- simulate --preset bau --output output.csv

# Custom time range and step
cargo run --bin world3-cli -- simulate --preset stabilized --start 1970 --end 2100 --dt 0.5
```

**CSV output columns (22 fields):**

```
year, population, cohort_0_14, cohort_15_44, cohort_45_64, cohort_65_plus,
birth_rate, death_rate, life_expectancy, fertility_rate,
industrial_capital, service_capital, industrial_output, industrial_output_per_capita, service_output_per_capita,
arable_land, food, food_per_capita, land_yield,
nnr_fraction, persistent_pollution, pollution_index
```

### `validate`

Validate the BAU run against Meadows 1972 reference checkpoints. Checks qualitative dynamics (not exact values):

- 1900 population ~1.6B
- 1970 population ~3.6B
- Population peaks at 6B–12B between 2000–2070
- NNR fraction remaining in 2100 < 0.7
- Pollution index rises above 0.5 at some point

```bash
cargo run --bin world3-cli -- validate
```

### `presets`

List all available presets:

```bash
cargo run --bin world3-cli -- presets
```

## Available Presets

| Name | Description |
|------|-------------|
| `bau` | **Business as Usual** — Original World 3 standard run. No policy interventions. Reproduces Meadows et al. 1972 Fig. 35 dynamics. |
| `technology` | **Comprehensive Technology** — Resource efficiency 4×, strong pollution control (80%), improved agriculture, 75% family planning efficacy from 2000. No social or behavioral changes. |
| `stabilized` | **Stabilized World** — Full combination of technology, pollution control, family planning (95% from 1975), land protection, and resource efficiency. Closest to a sustainable trajectory in Meadows et al. |

## Architecture Overview

```
crates/
  world3-core/        [IMPLEMENTED] Pure simulation engine. WorldState, ScenarioParams,
                      5 sector ODEs (population, capital, agriculture, resources, pollution),
                      RK4 solver, lookup tables.
  world3-cli/         [IMPLEMENTED] Batch simulation and validation CLI.
  world3-api/         [PLANNED] Axum HTTP + WebSocket server (Phase 3).
  world3-ingestion/   [PLANNED] Live data pipeline — World Bank, NOAA, FAO, UN, BP (Phase 4).
frontend/             [PLANNED] SvelteKit + D3 charts, parameter sliders (Phase 5).
data/
  lookup_tables/      World 3 piecewise-linear tables (JSON). Required at runtime.
  presets/            Named scenario parameter sets (BAU, Technology, Stabilized).
docs/
  world3_equations.md All differential equations with references to original model.
```

## Development Phases

- [x] **Phase 1** — Core simulation engine: `world3-core`, `world3-cli`, 5 original World 3 sectors, RK4 solver, validation
- [ ] **Phase 2** — Modern extensions + calibration: 4 extension sectors (climate, energy, biodiversity, inequality), historical CSV calibration
- [ ] **Phase 3** — API server: Axum REST + WebSocket, streaming simulation
- [ ] **Phase 4** — Live data ingestion: 7 data sources, SQLite cache, broadcast
- [ ] **Phase 5** — Frontend: SvelteKit + D3, stores, parameter sliders, scenario comparison
- [ ] **Phase 6** — Polish + deployment: benchmarks, sensitivity analysis, Docker Compose, CI

## License

GPL v3 — see [LICENSE](LICENSE).
