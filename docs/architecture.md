# Macroco — System Architecture

> System design reference. Each section links to requirements in `docs/product-requirements.md`.

## Overview

Macroco is an online macroeconomic simulator based on the World 3 system dynamics model (Meadows et al., *Limits to Growth*), extended with modern indicators for climate, energy mix, biodiversity, and inequality. The system comprises four Rust crates (`world3-core`, `world3-api`, `world3-cli`, `world3-ingestion`) and a SvelteKit/TypeScript frontend with D3 v7 charts. The backend uses Axum for HTTP and WebSocket serving. Deployment targets Fly.io via a multi-stage Docker build.

## Component Map

| Component | Directory | Purpose | Implements |
|-----------|-----------|---------|------------|
| Simulation Engine | `crates/world3-core/` | Pure simulation: WorldState, sector ODEs, RK4 solver, lookup tables | REQ-001, REQ-002, REQ-004, REQ-005 |
| API Server | `crates/world3-api/` | Axum HTTP + WebSocket server, historical data API | REQ-007, REQ-008, REQ-012 |
| CLI | `crates/world3-cli/` | Batch simulation, validation, debugging diagnostics, historical calibration tests | REQ-037, REQ-038, REQ-039, REQ-040, REQ-026 |
| Data Pipeline | `crates/world3-ingestion/` | Live data from external APIs, SQLite cache | REQ-013, REQ-014 |
| Frontend | `frontend/` | SvelteKit + D3 interactive UI | REQ-009, REQ-020--REQ-025, REQ-037 |
| Deployment | `/` (Dockerfile, fly.toml) | Containerized deployment on Fly.io | REQ-016 |
| CI/CD | `.github/workflows/` | Automated testing and deployment | REQ-010, REQ-017 |

## Simulation Engine (`world3-core`)

Implements: REQ-001, REQ-002, REQ-004, REQ-005

- `WorldState` is a typed struct (not `Vec<f64>`) with fields mirroring the published World 3 equations directly.
- `to_vec()` / `from_vec()` on `WorldState` are used only at solver boundaries for RK4 arithmetic.
- Sector derivative evaluation order is fixed: resource_aux, capital, resource_depletion, agriculture, pollution, population. Defined in `derivatives.rs`.
- `WorldState::N` = 16 ODE stocks. Adding or removing a stock requires updating: `N`, `to_vec()`, `from_vec()`, `Add`/`Mul` impls, `derivatives.rs`, and `initial_1900()`.
- `ScenarioParams::default()` matches the Collapse preset (`data/presets/collapse.json`).
- All non-linear relationships are encoded as `LookupTable` (piecewise-linear interpolation). Tables are loaded from `data/lookup_tables/*.json` and audited against the pyworld3 reference (World3-03 Vensim).
- Simulation is CPU-bound; must be run via `tokio::task::spawn_blocking` to avoid blocking the async reactor.

### Validation Module

- `world3_core::validation::validate_collapse()` runs qualitative dynamics checks against a `SimulationOutput` and returns `Vec<CheckResult>`.
- Used by: `world3-core/tests/qualitative_dynamics.rs` (integration tests), CLI `validate` command (thin wrapper).
- Checks: population trajectory, NNR depletion, pollution peak, IOPC collapse, life expectancy decline.

## API Server (`world3-api`)

Implements: REQ-007, REQ-008, REQ-012

- `AppState` holds: solver, lookup tables, scenario store (`RwLock<HashMap<Uuid, Scenario>>`), live data snapshot, ingestion broadcast sender, and historical data.
- REST endpoints served at `/api/v1/*`.
- WebSocket sessions stream simulation steps via an `mpsc` channel from a blocking task to the async handler. Parameter updates are debounced 50 ms server-side; the current simulation task is aborted and replaced.
- Historical data is loaded once at startup from `data/historical/*.csv` into `HashMap<String, HistoricalVariable>`. The path is configurable via `HISTORICAL_DATA_DIR` env var.
- Historical API: `GET /api/v1/historical` (all variables) and `GET /api/v1/historical/{variable_id}`. Both return `Cache-Control: public, max-age=86400`.
- In production, serves the static frontend via `tower-http::ServeDir` with SPA fallback. The `STATIC_DIR` env var points to the frontend build output.
- Graceful shutdown handles SIGTERM and Ctrl-C with a 15-second drain timeout (fly.toml `kill_timeout = 20s`).

## Frontend (`frontend`)

Implements: REQ-009, REQ-020, REQ-021, REQ-022, REQ-023, REQ-024, REQ-025, REQ-037

- SvelteKit 2 with Svelte 5 runes (`$state`, `$derived`, `$effect`) for all reactivity. D3 v7 used directly (no chart library wrapper) for custom multi-axis, phase-plane, and animated transition patterns.
- `frontend/src/lib/env.ts` provides `getApiBase()` / `getWsBase()` -- returns relative URLs in production (empty `PUBLIC_*` vars), absolute URLs in dev mode.
- WebSocket client auto-reconnects with 2-second backoff. All messages are typed against `WsClientMsg` / `WsServerMsg`.
- Content single source of truth: `frontend/src/lib/content/variable-descriptions.ts` (variable and parameter descriptions).
- Chart annotations (peaks, thresholds) defined in `frontend/src/lib/content/chart-annotations.ts`.
- Info panels use composition: `InfoPanelShell` (chrome + Escape handler + expert toggle) wraps panel-specific content, which includes shared `FeedbackLoops` and `RelatedVars` sub-components.

### Chart Zoom (REQ-037)

- `UnifiedChart.svelte` uses `d3.zoom()` for X-axis zoom/pan on all devices (wheel, pinch, drag). Replaces the previous no-op brush.
- Performance: the main `$effect` reads `currentTransform` via `untrack()` to avoid reactive re-runs on zoom. An `applyZoomTransform` closure (captured from the effect) does lightweight DOM updates at ~60Hz without rebuilding D3 joins.
- Transform restore on effect re-run uses a feedback-loop prevention pattern: temporarily detach the zoom handler, set the transform, re-attach.
- Mobile tooltip uses tap-to-pin (`pointer: coarse` detection) instead of hover. Tap detection: elapsed < 300ms, distance < 10px.
- Pure helpers extracted to `frontend/src/lib/charts/zoom-helpers.ts`: `constrainToXAxis`, `isTransformZoomed`, `isTap`, `computeTrend`, `computeVisibleYExtent`.
- SVG clip path (per-instance unique ID) prevents line overflow when zoomed.

### Historical Data Overlay

Implements: REQ-012

- Pre-bundled CSVs in `data/historical/` with provenance comment headers (source, URL, units, transformation, retrieval date).
- Six variables: `population`, `resources`, `food`, `industrial`, `pollution`, `life-expectancy`.
- Variable IDs must match the frontend's `unified-config.ts` configuration.
- Data flow: CSV files loaded at backend startup, served via REST API, fetched into `historicalStore` on the frontend, rendered as dashed D3 overlay lines.
- In normalized mode, combined min/max normalization (union of simulation and historical ranges) keeps both lines within [0, 1].
- Toggle in chart legend, default on.

### Parameter Explanation Panels

Implements: REQ-020 (extension)

- Extended `ParameterInfo` interface adds `feedbackLoops`, `relatedVariables`, and `impact` (increase/decrease text + sparkline variable) fields.
- Stores: `selectedParameterId` (writable, mutually exclusive with `selectedVariableId`) and `highlightedVariables` (derived from selected parameter's related variables).
- `ParameterInfoPanel`: 340px fixed-right slide-in panel with sparkline (Collapse vs. current scenario), impact cards, feedback loops, and related variables.
- Chart highlighting: when `highlightedVariables` is non-empty, related lines render at full opacity with thicker stroke; all other lines dim to 0.15 opacity.
- Info icon on `ParameterSlider` triggers the panel.

## Data Pipeline (`world3-ingestion`)

Implements: REQ-013, REQ-014

- `DataSource` trait: each source implements `fetch() -> RawSourceData` and declares its `update_interval`.
- Fallback chain: live API, then SQLite disk cache, then bundled historical CSV. Never fails silently.
- `mapping.rs` is the single source of truth for translating real-world observations into `WorldState` initial conditions.

## CLI (`world3-cli`)

Implements: REQ-037, REQ-038, REQ-039, REQ-040

The CLI serves four roles:
1. **CI/CD validation** (REQ-037): `validate` is a thin wrapper around `world3_core::validation::validate_collapse()` — prints PASS/FAIL, exits 1 on failure.
2. **Batch export** (REQ-038): `simulate --output` exports 25-column CSV for external analysis.
3. **Simulation debugging** (REQ-039): `diagnose` produces structured text/JSON reports (peaks, phases, anomalies, oscillation detection, dt-sensitivity, preset comparison).
4. **Reproducibility** (REQ-040): `simulate --preset` and `presets` provide deterministic named runs.

All scenario tests (qualitative dynamics, historical calibration) live in `world3-core/tests/`. The CLI has no integration tests — it only contains CLI-specific code.

### Historical Calibration Tests (REQ-026)

- Integration test in `crates/world3-core/tests/historical_calibration.rs` compares Collapse simulation against real-world historical CSVs.
- Metric: RMSE as percentage of mean historical value, computed over overlapping years (~1960-2023).
- Four variables tested: Population (<16%), Food/capita (<22%), IOPC (<23%), NNR fraction (<15%).
- Shared `OnceLock<SimulationOutput>` in `tests/common/mod.rs` avoids redundant Collapse simulation runs across tests.
- Historical CSVs loaded with a minimal inline parser (no dependency on `world3-api`'s parser). Uses `CARGO_MANIFEST_DIR` to resolve `data/historical/` path.
- Run: `cargo test -p world3-core --test historical_calibration` (summary only); `-- --nocapture` for full report.

## Deployment

Implements: REQ-016

- Multi-stage Dockerfile: Rust builder, Node builder, slim runtime image.
- Fly.io deployment in `cdg` region with auto-stop enabled.
- `STATIC_DIR` env var configures the path to frontend build output.
- `fly.toml`: `kill_timeout = 20s` allows graceful shutdown to complete.

## CI/CD

Implements: REQ-010, REQ-017

- GitHub Actions pipeline: clippy, test, frontend-test, traceability, deploy (on push to main).
- PR preview deploy: add `deploy-preview` label to any PR to deploy to Fly.io; removed/merged/closed auto-reverts to main. Comment steps use `continue-on-error` to tolerate transient GitHub API failures.
- Frontend tests: vitest + jsdom, run on every PR.
- Deploy job gated on `environment: production` with required status checks. Preview deploys use `environment: preview`.
- Branch ruleset on main: PR required (1 approval), rebase-only merge, linear history, no force push.
