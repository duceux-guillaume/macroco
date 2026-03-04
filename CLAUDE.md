# Macroco — Developer Guide

## Project Overview
Online live macroeconomic model based on the World 3 system dynamics model (Meadows et al., *Limits to Growth*). Extended with modern indicators: climate, energy mix, biodiversity, inequality.

**Stack:** Rust backend (Axum) + SvelteKit/TypeScript frontend + D3 v7. Deployed on Fly.io.

## Current Objective
Milestone 1: Interactive Limits to Growth. The frontend must be a self-contained
educational experience. Every chart variable has a human-readable explanation.
Users can change assumptions and immediately understand consequences.

Priority order:
1. Quick start guide (docs/quick-start.md + run.sh)
2. Model guide documentation (docs/model-guide.md)
3. Variable description content module (frontend/src/lib/content/)
4. Rich chart tooltips
5. Variable info panels
6. Simulation controls
7. Chart annotations
8. Preset comparison UX

Design principles:
- Documentation-first: every variable, parameter, and feedback loop has beginner + expert explanation
- Frontend content from single source of truth (variable-descriptions.ts)
- No backend changes needed for UX enhancements
- Maintain dark theme, D3 direct rendering, existing component patterns

## Repository Structure

```
crates/
  world3-core/        # Pure simulation engine (no I/O). WorldState, ScenarioParams, sector ODEs, RK4 solver.
  world3-api/         # Axum HTTP + WebSocket server. REST endpoints + streaming simulation.
  world3-ingestion/   # Live data pipeline. Fetches World Bank, NOAA, FAO, UN, BP. SQLite cache.
  world3-cli/         # Batch simulation / validation CLI.
frontend/             # SvelteKit app. D3 charts, parameter sliders, scenario management.
  src/lib/content/    # Variable descriptions, chart annotations — single source of truth for all UI text.
data/
  lookup_tables/      # World 3 piecewise-linear tables (JSON). Must be present at runtime.
  historical/         # Bundled historical CSVs used as seed/fallback data.
  presets/            # Named scenario parameter sets (BAU, Technology, Stabilized, LtG 1972).
docs/
  quick-start.md       # Beginner-friendly setup guide
  model-guide.md       # World 3 model explanation (beginner + expert tracks)
  simulation-engine.md # World 3 model architecture, sectors, solver
  cli.md               # CLI commands and flags reference
  api-server.md        # REST + WebSocket API documentation
  chart-output.md      # PNG chart rendering feature
  deployment.md        # Fly.io deployment guide
  examples/            # Generated example charts
Dockerfile            # Multi-stage build (Rust + Node + slim runtime)
fly.toml              # Fly.io app configuration
.claude/commands/      # Project-local Claude Code slash commands (/audit-tables)
```

## Commands

```bash
# NOTE: Dockerfile uses rust:1.85+ (required for edition2024 transitive deps)
# Quick start — builds frontend, serves everything on http://localhost:8080
./run.sh

# Quick start — frontend hot-reload mode (backend :8080 + Vite :5173)
./run.sh --dev

# Build everything
cargo build --workspace

# Run simulation CLI
cargo run --bin world3-cli -- simulate --preset bau --output output.csv

# Validate against Meadows 1972 reference trajectories
cargo run --bin world3-cli -- validate

# Run API server (serving static frontend)
STATIC_DIR=frontend/build RUST_LOG=debug cargo run --bin world3-api

# Run frontend dev server (separate terminal, for hot-reload)
cd frontend && npm run dev

# Docker build and run
docker build -t macroco .
docker run -p 8080:8080 macroco

# Deploy to Fly.io
flyctl deploy --remote-only

# Tests
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd frontend && npm run check && npm test   # vitest unit tests
cd frontend && npm run test:watch          # vitest in watch mode
```

## Key Architecture Decisions

### Simulation Engine (`world3-core`)
- `WorldState` is a typed struct (not `Vec<f64>`) — fields mirror published World 3 equations directly.
- `to_vec()` / `from_vec()` on `WorldState` are used only at solver boundaries (RK4 arithmetic).
- Sector derivative order matters: resource_aux → capital → resource_depletion → agriculture → pollution → population. Documented in `derivatives.rs`.
- `WorldState::N` = 16 ODE stocks. When adding/removing stocks, update: `N`, `to_vec()`, `from_vec()`, `Add`/`Mul` impls, `derivatives.rs` (assembly + doc comments), and `initial_1900()`.
- `ScenarioParams::default()` must match BAU preset. When changing defaults, also update `data/presets/business_as_usual.json`.
- All non-linear relationships encoded as `LookupTable` (piecewise-linear). Tables loaded from `/data/lookup_tables/*.json`.
- Lookup tables in `crates/world3-core/src/lookup/tables.rs` are audited against pyworld3 reference (World3-03 Vensim). See `docs/audit.md`. Run `/audit-tables` to re-audit after changes.
- pyworld3 reference: `https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json`
- Simulation is CPU-bound; always run via `tokio::task::spawn_blocking` to avoid blocking the async reactor.

### API Server (`world3-api`)
- `AppState` holds: solver, lookup tables, scenario store (`RwLock<HashMap<Uuid, Scenario>>`), live data snapshot, ingestion broadcast sender.
- WebSocket sessions stream simulation steps via `mpsc` channel from blocking task to async handler.
- Parameter updates from the frontend are debounced 50ms server-side; current task is aborted and replaced.
- In production, serves static frontend via `tower-http::ServeDir` with SPA fallback (`STATIC_DIR` env var). API at `/api/v1/*`, frontend at all other paths.
- Graceful shutdown handles SIGTERM + Ctrl-C with 15s drain timeout (fly.toml `kill_timeout = 20s`).

### Data Ingestion (`world3-ingestion`)
- `DataSource` trait: each source implements `fetch() → RawSourceData` and declares its `update_interval`.
- Fallback chain: live API → SQLite disk cache → bundled historical CSV. Never fails silently.
- `mapping.rs` is the single source of truth for translating real-world observations into `WorldState` initial conditions.

### Frontend
- `frontend/src/lib/env.ts` provides `getApiBase()` / `getWsBase()` — returns relative URLs in production (empty `PUBLIC_*` vars), absolute URLs in dev. All API/WS imports use this, not `$env/static/public` directly.
- `frontend/.env.production` has empty `PUBLIC_API_BASE=` and `PUBLIC_WS_BASE=` to trigger same-origin fallback.
- Svelte reactive stores (`$:`) drive all chart updates — avoid imperative D3 re-render calls outside the reactive block.
- D3 is used directly (not wrapped in a chart library) because World 3 output requires custom multi-axis, phase-plane, and animated transition patterns.
- WS client auto-reconnects with 2s backoff. All WS messages are typed against `WsClientMsg` / `WsServerMsg`.
- All variable/parameter descriptions live in `frontend/src/lib/content/variable-descriptions.ts` — single source of truth.
- Chart annotations (peaks, thresholds) defined in `frontend/src/lib/content/chart-annotations.ts`.

### Frontend Testing
- Test stack: vitest + jsdom. Config in `vite.config.ts` (import `defineConfig` from `vitest/config`, not `vite`).
- Test helpers in `frontend/src/lib/test-helpers.ts` — `makeWorldState()` and `makeTimeSeries()` factories.
- Mock SvelteKit env: `vi.mock('$env/static/public', () => ({ PUBLIC_API_BASE: '...' }))` before imports.
- Svelte stores in tests: use `get()` from `svelte/store`; reset writable stores in `beforeEach`.

### Backend Testing
- `approx` crate `assert_relative_eq!` does NOT support format string messages — use separate `assert!` for custom messages.
- Run full verification after model changes: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo run --bin world3-cli -- validate`

## Model Sectors (5 — original World 3)
Population · Industrial Capital · Agriculture · Non-Renewable Resources · Pollution

Future extensions (Milestone 2): Climate (CO₂/EBM temperature) · Energy Mix · Biodiversity (LPI) · Inequality (Gini/HDI)

## Environment Variables

```env
# Backend
RUST_LOG=info,world3_api=debug
STATIC_DIR=./static    # path to frontend build output (default: ./static)
DATABASE_URL=sqlite:///data/cache.db
CORS_ORIGINS=http://localhost:5173
FAO_API_KEY=           # optional — FAO FAOSTAT
IEA_API_KEY=           # optional — IEA detailed energy data

# Frontend (dev mode only — production uses same-origin relative URLs)
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

## CI/CD
- GitHub Actions: clippy → test → frontend-test → deploy (on push to main only)
- Deploy gated on `environment: production` with required status checks
- `frontend-test` job runs: `npm run check`, `npm test`, `npm run build`
- Ruleset on main: PR required (1 approval), rebase-only, linear history, no force push

## Product Milestones
1. **Milestone 1 — Interactive Limits to Growth** (current): Documentation, tooltips, info panels, simulation controls, annotations, preset comparison.
2. **Milestone 2 — Modern Extensions**: 4 extension sectors (climate, energy, biodiversity, inequality), historical CSV calibration.
3. **Milestone 3 — Live Data Pipeline**: `world3-ingestion` crate, 7 data sources, SQLite cache, broadcast.
4. **Milestone 4 — Production Deployment**: Fly.io, CI/CD, benchmarks, sensitivity analysis.

## License
GPL v3 — see LICENSE file.
