# Macroco — Developer Guide

## Project Overview
Online live macroeconomic model based on the World 3 system dynamics model (Meadows et al., *Limits to Growth*). Extended with modern indicators: climate, energy mix, biodiversity, inequality.

**Stack:** Rust backend (Axum) + SvelteKit/TypeScript frontend + D3 v7. Deployed via Docker Compose.

## Repository Structure

```
crates/
  world3-core/        # Pure simulation engine (no I/O). WorldState, ScenarioParams, sector ODEs, RK4 solver.
  world3-api/         # Axum HTTP + WebSocket server. REST endpoints + streaming simulation.
  world3-ingestion/   # Live data pipeline. Fetches World Bank, NOAA, FAO, UN, BP. SQLite cache.
  world3-cli/         # Batch simulation / validation CLI.
frontend/             # SvelteKit app. D3 charts, parameter sliders, scenario management.
data/
  lookup_tables/      # World 3 piecewise-linear tables (JSON). Must be present at runtime.
  historical/         # Bundled historical CSVs used as seed/fallback data.
  presets/            # Named scenario parameter sets (BAU, Technology, Stabilized, LtG 1972).
docs/
  world3_equations.md # All differential equations with references to original model.
  api_reference.md    # REST + WebSocket API documentation.
```

## Commands

```bash
# Build everything
cargo build --workspace

# Run simulation CLI (Phase 1 validation)
cargo run --bin world3-cli -- simulate --preset bau --output output.csv

# Validate against Meadows 1972 reference trajectories
cargo run --bin world3-cli -- validate

# Run API server (development)
RUST_LOG=debug cargo run --bin world3-api

# Run frontend (development, separate terminal)
cd frontend && npm run dev

# Full stack via Docker Compose
docker compose up

# Tests
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && npm run check && npm test
```

## Key Architecture Decisions

### Simulation Engine (`world3-core`)
- `WorldState` is a typed struct (not `Vec<f64>`) — fields mirror published World 3 equations directly.
- `to_vec()` / `from_vec()` on `WorldState` are used only at solver boundaries (RK4 arithmetic).
- Sector derivative order matters: resources → capital → agriculture → population → pollution → extensions.
- All non-linear relationships encoded as `LookupTable` (piecewise-linear). Tables loaded from `/data/lookup_tables/*.json`.
- Simulation is CPU-bound; always run via `tokio::task::spawn_blocking` to avoid blocking the async reactor.

### API Server (`world3-api`)
- `AppState` holds: solver, lookup tables, scenario store (`RwLock<HashMap<Uuid, Scenario>>`), live data snapshot, ingestion broadcast sender.
- WebSocket sessions stream simulation steps via `mpsc` channel from blocking task to async handler.
- Parameter updates from the frontend are debounced 50ms server-side; current task is aborted and replaced.

### Data Ingestion (`world3-ingestion`)
- `DataSource` trait: each source implements `fetch() → RawSourceData` and declares its `update_interval`.
- Fallback chain: live API → SQLite disk cache → bundled historical CSV. Never fails silently.
- `mapping.rs` is the single source of truth for translating real-world observations into `WorldState` initial conditions.

### Frontend
- Svelte reactive stores (`$:`) drive all chart updates — avoid imperative D3 re-render calls outside the reactive block.
- D3 is used directly (not wrapped in a chart library) because World 3 output requires custom multi-axis, phase-plane, and animated transition patterns.
- WS client auto-reconnects with 2s backoff. All WS messages are typed against `WsClientMsg` / `WsServerMsg`.

## Model Sectors (9 total)
Original World 3: Population · Industrial Capital · Agriculture · Non-Renewable Resources · Pollution
Extensions: Climate (CO₂/EBM temperature) · Energy Mix · Biodiversity (LPI) · Inequality (Gini/HDI)

## Environment Variables

```env
# Backend
RUST_LOG=info,world3_api=debug
DATABASE_URL=sqlite:///data/cache.db
CORS_ORIGINS=http://localhost:5173
FAO_API_KEY=           # optional — FAO FAOSTAT
IEA_API_KEY=           # optional — IEA detailed energy data

# Frontend
PUBLIC_API_BASE=http://localhost:8080/api/v1
PUBLIC_WS_BASE=ws://localhost:8080/api/v1/ws
```

## Validation Baseline
The "standard run" (BAU preset, 1900–2100, no policy interventions) must reproduce Meadows 1972 Fig. 35 dynamics:
- Global population peaks ~2030 at ~8B then declines
- Non-renewable resources fall to ~50% of initial by ~2050
- Food per capita peaks mid-century then falls
- Industrial output per capita peaks and collapses before 2100

Run `cargo run --bin world3-cli -- validate` to check against bundled reference trajectories.

## Development Phases
1. **Phase 1 — Core Simulation Engine** (current): Rust workspace, `world3-core`, `world3-cli`, all 5 original World 3 sectors, RK4 solver, validation.
2. **Phase 2 — Modern Extensions + Calibration**: 4 extension sectors, historical CSV calibration, preset scenarios.
3. **Phase 3 — API Server**: Axum REST + WebSocket, `spawn_blocking`, streaming simulation.
4. **Phase 4 — Live Data Ingestion**: `world3-ingestion` crate, 7 data sources, SQLite cache, broadcast.
5. **Phase 5 — Frontend**: SvelteKit + D3, stores, parameter sliders, scenario comparison.
6. **Phase 6 — Polish + Deployment**: Benchmarks, sensitivity analysis, Docker Compose, CI.

## License
GPL v3 — see LICENSE file.
