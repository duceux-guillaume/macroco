# Macroco — Product Requirements

> Living document managed by the `/pm` skill. Each requirement has a stable ID (REQ-NNN).

---

## Done

- [x] **REQ-001: Core simulation engine**
  - *Context:* The World 3 model requires a typed `WorldState` struct with sector-level ODEs and an RK4 solver to reproduce Meadows 1972 dynamics.
  - *Done:* `world3-core` crate implements WorldState, 5 original sectors (population, capital, agriculture, resources, pollution), and RK4 solver.

- [x] **REQ-002: Lookup table infrastructure**
  - *Context:* World 3 non-linear relationships are encoded as piecewise-linear tables.
  - *Done:* `LookupTable` type in `world3-core`; tables hardcoded in `world3-core/src/lookup/tables.rs`.

- [x] **REQ-003: CLI batch simulation**
  - *Context:* Users need a command-line tool to run simulations, validate against reference trajectories, and export results.
  - *Done:* `world3-cli` crate with `simulate`, `validate`, and `presets` subcommands; CSV output support.

- [x] **REQ-004: Scenario presets**
  - *Context:* Named parameter sets allow reproducible runs and comparison against canonical scenarios.
  - *Done:* Three presets in `data/presets/`: Business as Usual, Comprehensive Technology, Stabilized World.

- [x] **REQ-005: Validation against Meadows 1972**
  - *Context:* The standard BAU run must reproduce the qualitative dynamics of Meadows 1972 Fig. 35.
  - *Done:* `world3-cli validate` checks population peak, resource depletion, food and industrial output trajectories.

- [x] **REQ-006: PNG chart output**
  - *Context:* Visual output of simulation results is needed for documentation and quick inspection.
  - *Done:* `--chart` flag on `world3-cli simulate` renders multi-panel PNG via the `plotters` crate.

- [x] **REQ-007: REST API server**
  - *Context:* The frontend and external clients need HTTP endpoints for scenario CRUD, simulation runs, and parameter schema.
  - *Done:* `world3-api` crate with Axum REST endpoints, CORS support, and scenario management.

- [x] **REQ-008: WebSocket streaming simulation**
  - *Context:* Real-time parameter adjustment requires streaming simulation steps to the frontend with low latency.
  - *Done:* WebSocket handler in `world3-api` with `mpsc` channel from `spawn_blocking` task; 50ms server-side debounce.

- [x] **REQ-009: SvelteKit frontend**
  - *Context:* An interactive web UI is needed for exploring scenarios, adjusting parameters, and viewing real-time simulation output.
  - *Done:* SvelteKit 2 + Svelte 5 app with D3 v7 charts, parameter sliders, scenario bar, and WebSocket auto-reconnect.

- [x] **REQ-010: CI pipeline**
  - *Context:* Automated quality checks prevent regressions in the Rust workspace.
  - *Done:* GitHub Actions workflow running `cargo clippy` and `cargo test` on PRs and pushes to main.

- [x] **REQ-018: Quick start guide and run script**
  - *Context:* New users need a beginner-friendly way to launch the full stack without reading the full README.
  - *Done:* `docs/quick-start.md` with platform-specific install steps and troubleshooting; `run.sh` launches backend + frontend in parallel with signal handling.

- [x] **REQ-019: Model guide documentation**
  - *Context:* Users need to understand World 3 sectors, feedback loops, and how to interpret charts before exploring scenarios.
  - *Done:* `docs/model-guide.md` with beginner + technical tracks covering all 5 sectors, 6 feedback loops, solver explanation, and preset comparison.

- [x] **REQ-020: Variable descriptions content module**
  - *Context:* All UI text describing variables, parameters, and feedback loops must come from a single source of truth.
  - *Done:* `frontend/src/lib/content/variable-descriptions.ts` — 16 variables, 12 parameters, 6 feedback loops with beginner + expert descriptions.

- [x] **REQ-021: Rich chart tooltips**
  - *Context:* Hovering on charts should show year, values per scenario, trend indicators, and a short variable description.
  - *Done:* Tooltip overlay in `TimeSeriesChart.svelte` with vertical guide line, per-scenario values, trend arrows, and description hint.

- [x] **REQ-022: Variable info panels**
  - *Context:* Users need on-demand deep explanations of any variable — beginner and expert level — accessible directly from charts.
  - *Done:* `VariableInfoPanel.svelte` slide-in panel triggered by chart title click; shows sector, unit, descriptions, feedback loops, and related variables.

- [x] **REQ-023: Simulation controls**
  - *Context:* Users should be able to adjust simulation time range and resolution without editing code.
  - *Done:* `SimulationControls.svelte` with start year, end year, and time step dropdowns; 200ms debounce on WebSocket updates.

- [x] **REQ-024: Chart annotations**
  - *Context:* Charts should highlight key events (historical markers, peaks, threshold crossings) to guide interpretation.
  - *Done:* `frontend/src/lib/content/chart-annotations.ts` with static + dynamic annotations; rendered as dashed vertical lines with rotated labels in D3.

- [x] **REQ-025: Preset comparison UX**
  - *Context:* Users should be able to overlay multiple scenarios on the same charts and toggle visibility.
  - *Done:* `ScenarioBar` with click-to-toggle and double-click-to-focus; `ScenarioSelector` with "Compare All" button and per-scenario descriptions.

---

## In Progress

*(No requirements currently in progress.)*

---

## Planned

- [ ] **REQ-011: Extension sectors — climate, energy, biodiversity, inequality**
  - *Context:* Milestone 2 extends the World 3 model with four modern sectors: Climate (CO2/EBM temperature), Energy Mix, Biodiversity (LPI), and Inequality (Gini/HDI).
  - *Priority:* high

- [ ] **REQ-012: Historical data calibration**
  - *Context:* Milestone 2 requires calibrating model parameters against real historical data (World Bank, NOAA, FAO, UN, BP) to improve predictive accuracy.
  - *Priority:* high

- [ ] **REQ-013: Live data ingestion pipeline**
  - *Context:* Milestone 3 introduces `world3-ingestion` with the `DataSource` trait, fetching live data from 7 external APIs with SQLite cache and fallback to bundled CSVs.
  - *Priority:* medium

- [ ] **REQ-014: Data mapping layer**
  - *Context:* A `mapping.rs` module must translate raw external observations into `WorldState` initial conditions, serving as the single source of truth for real-world-to-model conversion.
  - *Priority:* medium

- [ ] **REQ-015: Performance benchmarks and sensitivity analysis**
  - *Context:* Milestone 4 requires benchmarks for solver performance and sensitivity analysis to quantify how parameter changes affect model outputs.
  - *Priority:* medium

- [ ] **REQ-016: Docker Compose deployment**
  - *Context:* Milestone 4 targets a single `docker compose up` command to run the full stack (API server + frontend + ingestion).
  - *Priority:* medium

- [ ] **REQ-017: Frontend test suite**
  - *Context:* The frontend currently has no tests (`package.json` test script is a placeholder). Svelte component and integration tests are needed.
  - *Priority:* low
