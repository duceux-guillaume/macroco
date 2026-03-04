# Macroco — Product Requirements

> Living document. Each requirement has a stable ID (REQ-NNN). See `docs/architecture.md` for component design.

---

## Done

- [x] **REQ-001: Core simulation engine**
  - *Context:* The World 3 model requires a typed `WorldState` struct with sector-level ODEs and an RK4 solver to reproduce Meadows 1972 dynamics.
  - *Components:* `world3-core`: WorldState, sector ODEs, RK4 solver
  - *Done:* `world3-core` crate implements WorldState, 5 original sectors (population, capital, agriculture, resources, pollution), and RK4 solver.

- [x] **REQ-002: Lookup table infrastructure**
  - *Context:* World 3 non-linear relationships are encoded as piecewise-linear tables.
  - *Components:* `world3-core`: LookupTable, `data/lookup_tables/`
  - *Done:* `LookupTable` type in `world3-core`; tables hardcoded in `world3-core/src/lookup/tables.rs`.

- [x] **REQ-003: CLI batch simulation**
  - *Context:* Users need a command-line tool to run simulations, validate against reference trajectories, and export results.
  - *Components:* `world3-cli`: simulate, validate, presets subcommands
  - *Done:* `world3-cli` crate with `simulate`, `validate`, and `presets` subcommands; CSV output support.

- [x] **REQ-004: Scenario presets**
  - *Context:* Named parameter sets allow reproducible runs and comparison against canonical scenarios.
  - *Components:* `world3-core`: ScenarioParams, `data/presets/`
  - *Done:* Three presets (Business as Usual, Comprehensive Technology, Stabilized World) constructed in Rust code; JSON copies in `data/presets/`.

- [x] **REQ-005: Validation against Meadows 1972**
  - *Context:* The standard BAU run must reproduce the qualitative dynamics of Meadows 1972 Fig. 35.
  - *Components:* `world3-cli`: validate subcommand
  - *Done:* `world3-cli validate` checks population peak, resource depletion, food and industrial output trajectories.

- [x] **REQ-006: PNG chart output**
  - *Context:* Visual output of simulation results is needed for documentation and quick inspection.
  - *Components:* `world3-cli`: `--chart` flag, plotters crate
  - *Done:* `--chart` flag on `world3-cli simulate` renders multi-panel PNG via the `plotters` crate.

- [x] **REQ-007: REST API server**
  - *Context:* The frontend and external clients need HTTP endpoints for scenario CRUD, simulation runs, and parameter schema.
  - *Components:* `world3-api`: Axum REST endpoints, scenario store
  - *Done:* `world3-api` crate with Axum REST endpoints, CORS support, and scenario management.

- [x] **REQ-008: WebSocket streaming simulation**
  - *Context:* Real-time parameter adjustment requires streaming simulation steps to the frontend with low latency.
  - *Components:* `world3-api`: WebSocket handler, mpsc channel; `frontend`: WS client
  - *Done:* WebSocket handler in `world3-api` with `mpsc` channel from `spawn_blocking` task; 50ms server-side debounce.

- [x] **REQ-009: SvelteKit frontend**
  - *Context:* An interactive web UI is needed for exploring scenarios, adjusting parameters, and viewing real-time simulation output.
  - *Components:* `frontend`: SvelteKit app, D3 charts, parameter sliders, scenario bar
  - *Done:* SvelteKit 2 + Svelte 5 app with D3 v7 charts, parameter sliders, scenario bar, and WebSocket auto-reconnect.

- [x] **REQ-010: CI pipeline**
  - *Context:* Automated quality checks prevent regressions in the Rust workspace.
  - *Components:* `.github/workflows/`: clippy + test jobs
  - *Done:* GitHub Actions workflow running `cargo clippy` and `cargo test` on PRs and pushes to main.
  - *Exempt:* CI infrastructure, validated by pipeline execution

- [x] **REQ-012: Historical data calibration**
  - *Context:* Calibrating model parameters against real historical data (World Bank, FAO, OWID) improves predictive accuracy and validates the simulation against observed trends.
  - *Components:* `world3-api`: historical.rs, CSV parser, `/api/v1/historical`; `frontend`: historicalStore, chart overlays; `data/historical/*.csv`
  - *Done:* Historical data API and frontend overlay implemented with bundled CSVs.

- [x] **REQ-016: Containerized deployment**
  - *Context:* Single-container deployment on Fly.io. The `world3-api` binary serves both API and static frontend.
  - *Components:* Dockerfile, fly.toml, CI/CD deploy job, `world3-api` static serving
  - *Done:* Multi-stage Dockerfile, `fly.toml` config (cdg region, auto-stop), CI/CD deploy job in GitHub Actions, graceful shutdown, `STATIC_DIR` env var for static file serving.
  - *Exempt:* Deployment infrastructure, validated by deploy job

- [x] **REQ-017: Frontend test suite**
  - *Context:* Vitest unit test suite: 10 test files, ~170 tests covering format/extract utils, chart configs, variable descriptions, annotations, stores, API client, and WebSocket client. CI pipeline runs tests on every PR.
  - *Components:* `frontend`: vitest suite, test-helpers.ts, CI frontend-test job
  - *Done:* Vitest test suite with CI integration.
  - *Exempt:* Meta-requirement; test suite existence is the implementation

- [x] **REQ-018: Quick start guide and run script**
  - *Context:* New users need a beginner-friendly way to launch the full stack without reading the full README.
  - *Components:* `docs/quick-start.md`, `run.sh`
  - *Done:* `docs/quick-start.md` with platform-specific install steps and troubleshooting; `run.sh` launches backend + frontend in parallel with signal handling.
  - *Exempt:* Documentation only

- [x] **REQ-019: Model guide documentation**
  - *Context:* Users need to understand World 3 sectors, feedback loops, and how to interpret charts before exploring scenarios.
  - *Components:* `docs/model-guide.md`
  - *Done:* `docs/model-guide.md` with beginner + technical tracks covering all 5 sectors, 6 feedback loops, solver explanation, and preset comparison.
  - *Exempt:* Documentation only

- [x] **REQ-020: Variable descriptions content module**
  - *Context:* All UI text describing variables, parameters, and feedback loops must come from a single source of truth.
  - *Components:* `frontend/src/lib/content/variable-descriptions.ts`, ParameterInfoPanel, ParameterSlider info icon
  - *Done:* `frontend/src/lib/content/variable-descriptions.ts` — 16 variables, 12 parameters, 6 feedback loops with beginner + expert descriptions.

- [x] **REQ-021: Rich chart tooltips**
  - *Context:* Hovering on charts should show year, values per scenario, trend indicators, and a short variable description.
  - *Components:* `frontend`: TimeSeriesChart tooltip overlay
  - *Done:* Tooltip overlay in `TimeSeriesChart.svelte` with vertical guide line, per-scenario values, trend arrows, and description hint.

- [x] **REQ-022: Variable info panels**
  - *Context:* Users need on-demand deep explanations of any variable — beginner and expert level — accessible directly from charts.
  - *Components:* `frontend`: VariableInfoPanel, InfoPanelShell
  - *Done:* `VariableInfoPanel.svelte` slide-in panel triggered by chart title click; shows sector, unit, descriptions, feedback loops, and related variables.

- [x] **REQ-023: Simulation controls**
  - *Context:* Users should be able to adjust simulation time range and resolution without editing code.
  - *Components:* `frontend`: SimulationControls component, WS debounce
  - *Done:* `SimulationControls.svelte` with start year, end year, and time step dropdowns; 200ms debounce on WebSocket updates.

- [x] **REQ-024: Chart annotations**
  - *Context:* Charts should highlight key events (historical markers, peaks, threshold crossings) to guide interpretation.
  - *Components:* `frontend/src/lib/content/chart-annotations.ts`, D3 annotation rendering
  - *Done:* `frontend/src/lib/content/chart-annotations.ts` with static + dynamic annotations; rendered as dashed vertical lines with rotated labels in D3.

- [x] **REQ-025: Preset comparison UX**
  - *Context:* Users should be able to overlay multiple scenarios on the same charts and toggle visibility.
  - *Components:* `frontend`: ScenarioBar, ScenarioSelector
  - *Done:* `ScenarioBar` with click-to-toggle and double-click-to-focus; `ScenarioSelector` with "Compare All" button and per-scenario descriptions.

- [x] **REQ-027: Bi-directional traceability**
  - *Context:* Coverage visibility and impact analysis require bidirectional links between requirements and tests.
  - *Components:* `scripts/traceability.py`, `docs/traceability-matrix.md`, CI traceability job
  - *Done:* `// REQ:` annotations in test files, CI script checks coverage, auto-generated traceability matrix.
  - *Exempt:* Infrastructure tooling; the traceability script itself is the implementation

- [x] **REQ-026: BAU historical calibration regression tests**
  - *Context:* The BAU simulation must track real-world historical data within RMSE% thresholds over ~1960-2023. Quantitative comparison against World Bank, FAO, and OWID data exposes calibration gaps.
  - *Components:* `world3-cli`: `tests/historical_calibration.rs`; `data/historical/*.csv`
  - *RMSE% thresholds:* Population (<16%), Food/capita (<22%), IOPC (<23%), NNR fraction (<15%)
  - *Max-year-error thresholds:* Population (<42%), Food/capita (<30%), IOPC (<43%), NNR fraction (<30%)
  - *Current metrics:* Pop RMSE=15.5%, Food RMSE=21.2%, IOPC RMSE=22.9%, NNR RMSE=7.1%. Max-year: Pop 41.5%@1961, Food 29.1%@2022, IOPC 42.3%@2023, NNR 29.9%@2023.
  - *Model changes:* ISOPC dynamic service demand reference (lookup table replaces hardcoded 200.0), technology_growth_rate=0.014 (was 0.002, compensates for real-world TFP ~1.5%/yr), FIOAC consumption fraction capped at 0.70 (was 0.83).
  - *Structural limitations:* Population 1960s overshoot (41.5% max error at 1961 -- model birth/death rates lag real demographic transition), IOPC post-2010 collapse (42.3% max error at 2023 -- BAU overshoot-collapse starts earlier than real world).
  - *Done:* All 4 RMSE + 4 max-year-error thresholds pass. 8 tests run in CI without `#[ignore]`.
  - *Design:* `docs/plans/2026-03-04-better-bau-calibration-design.md`

---

## In Progress

- [ ] **REQ-028: Multi-scenario historical calibration**
  - *Context:* All scenarios (Technology, Stabilized) must fit real-world historical data over the shared 1960–2023 period, not just BAU. Scenarios share the same history but diverge only after policy switch years.
  - *Components:* `world3-cli`: `tests/historical_calibration.rs`; `data/historical/*.csv`
  - *Priority:* high

- [ ] **REQ-029: Scenario trajectory validation**
  - *Context:* Non-BAU scenarios must produce meaningfully divergent futures compared to BAU. Technology should show extended resource availability and delayed decline; Stabilized should show population leveling off and sustained output. Validates that preset parameter differences actually produce distinct trajectories.
  - *Components:* `world3-cli`: validate subcommand or dedicated test
  - *Priority:* high

---

## Planned

- [ ] **REQ-011: Extension sectors — climate, energy, biodiversity, inequality**
  - *Context:* Milestone 2 extends the World 3 model with four modern sectors: Climate (CO2/EBM temperature), Energy Mix, Biodiversity (LPI), and Inequality (Gini/HDI).
  - *Priority:* high

- [ ] **REQ-013: Live data ingestion pipeline**
  - *Context:* Milestone 3 introduces `world3-ingestion` with the `DataSource` trait, fetching live data from 7 external APIs with SQLite cache and fallback to bundled CSVs.
  - *Priority:* medium

- [ ] **REQ-014: Data mapping layer**
  - *Context:* Milestone 3 requires a `mapping.rs` module to translate raw external observations into `WorldState` initial conditions, serving as the single source of truth for real-world-to-model conversion.
  - *Priority:* medium

- [ ] **REQ-015: Performance benchmarks and sensitivity analysis**
  - *Context:* Milestone 4 requires benchmarks for solver performance and sensitivity analysis to quantify how parameter changes affect model outputs.
  - *Priority:* medium

- [ ] **REQ-028: User feedback mechanism**
  - *Context:* Users need a way to report bugs and request features from within the app. A sidebar footer with GitHub Issues links provides a lightweight, zero-backend feedback channel.
  - *Priority:* medium
  - *Components:* `frontend/src/components/Sidebar.svelte`, `frontend/src/lib/utils/feedback-url.ts`, `.github/ISSUE_TEMPLATE/`
