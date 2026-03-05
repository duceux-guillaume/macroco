# Macroco — Product Requirements

> Living document. Each requirement has a stable ID (REQ-NNN) and a milestone tag.
> See `docs/architecture.md` for component design.
>
> **Milestones:** M1 Foundation · M2 Collapse · M3 Technotopia · M4 Ecotopia · M5 Living Data · M6 Deep Exploration

---

## Done

- [x] **REQ-001: Core simulation engine**
  - *Milestone:* M1
  - *Context:* The World 3 model requires a typed `WorldState` struct with sector-level ODEs and an RK4 solver to reproduce Meadows 1972 dynamics.
  - *Components:* `world3-core`: WorldState, sector ODEs, RK4 solver
  - *Done:* `world3-core` crate implements WorldState, 5 original sectors (population, capital, agriculture, resources, pollution), and RK4 solver.

- [x] **REQ-002: Lookup table infrastructure**
  - *Milestone:* M1
  - *Context:* World 3 non-linear relationships are encoded as piecewise-linear tables.
  - *Components:* `world3-core`: LookupTable, `data/lookup_tables/`
  - *Done:* `LookupTable` type in `world3-core`; tables hardcoded in `world3-core/src/lookup/tables.rs`.

- [x] **REQ-003: CLI batch simulation** *(Superseded by REQ-037..040)*
  - *Milestone:* M1
  - *Context:* Users need a command-line tool to run simulations, diagnose trajectories, and export results.
  - *Components:* `world3-cli`: simulate, diagnose, presets subcommands
  - *Superseded:* Split into four focused requirements: REQ-037 (CI/CD validation), REQ-038 (batch export), REQ-039 (simulation debugging), REQ-040 (reproducibility).

- [x] **REQ-004: Scenario presets**
  - *Milestone:* M1
  - *Context:* Named parameter sets allow reproducible runs and comparison against canonical scenarios.
  - *Components:* `world3-core`: ScenarioParams, `data/presets/`
  - *Done:* Three presets (Collapse, Comprehensive Technology, Stabilized World) constructed in Rust code; JSON copies in `data/presets/`.

- [x] **REQ-005: Validation against Meadows 1972**
  - *Milestone:* M2
  - *Context:* The standard Collapse run must reproduce the qualitative dynamics of Meadows 1972 Fig. 35.
  - *Components:* `world3-core`: `validation` module, `world3-cli`: `validate` subcommand (REQ-037)
  - *Done:* `world3-core::validate_collapse()` checks population peak, resource depletion, food and industrial output trajectories. CLI `validate` is a thin wrapper.

- [x] **REQ-007: REST API server**
  - *Milestone:* M1
  - *Context:* The frontend and external clients need HTTP endpoints for scenario CRUD, simulation runs, and parameter schema.
  - *Components:* `world3-api`: Axum REST endpoints, scenario store
  - *Done:* `world3-api` crate with Axum REST endpoints, CORS support, and scenario management.

- [x] **REQ-008: WebSocket streaming simulation**
  - *Milestone:* M1
  - *Context:* Real-time parameter adjustment requires streaming simulation steps to the frontend with low latency.
  - *Components:* `world3-api`: WebSocket handler, mpsc channel; `frontend`: WS client
  - *Done:* WebSocket handler in `world3-api` with `mpsc` channel from `spawn_blocking` task; 50ms server-side debounce.

- [x] **REQ-009: SvelteKit frontend**
  - *Milestone:* M1
  - *Context:* An interactive web UI is needed for exploring scenarios, adjusting parameters, and viewing real-time simulation output.
  - *Components:* `frontend`: SvelteKit app, D3 charts, parameter sliders, scenario bar
  - *Done:* SvelteKit 2 + Svelte 5 app with D3 v7 charts, parameter sliders, scenario bar, and WebSocket auto-reconnect.

- [x] **REQ-010: CI pipeline**
  - *Milestone:* M1
  - *Context:* Automated quality checks prevent regressions in the Rust workspace.
  - *Components:* `.github/workflows/`: clippy + test jobs
  - *Done:* GitHub Actions workflow running `cargo clippy` and `cargo test` on PRs and pushes to main.
  - *Exempt:* CI infrastructure, validated by pipeline execution

- [x] **REQ-012: Historical data calibration**
  - *Milestone:* M2
  - *Context:* Calibrating model parameters against real historical data (World Bank, FAO, OWID) improves predictive accuracy and validates the simulation against observed trends.
  - *Components:* `world3-api`: historical.rs, CSV parser, `/api/v1/historical`; `frontend`: historicalStore, chart overlays; `data/historical/*.csv`
  - *Done:* Historical data API and frontend overlay implemented with bundled CSVs.

- [x] **REQ-016: Containerized deployment**
  - *Milestone:* M1
  - *Context:* Single-container deployment on Fly.io. The `world3-api` binary serves both API and static frontend.
  - *Components:* Dockerfile, fly.toml, CI/CD deploy job, `world3-api` static serving
  - *Done:* Multi-stage Dockerfile, `fly.toml` config (cdg region, auto-stop), CI/CD deploy job in GitHub Actions, graceful shutdown, `STATIC_DIR` env var for static file serving.
  - *Exempt:* Deployment infrastructure, validated by deploy job

- [x] **REQ-017: Frontend test suite**
  - *Milestone:* M1
  - *Context:* Vitest unit test suite: 10 test files, ~170 tests covering format/extract utils, chart configs, variable descriptions, annotations, stores, API client, and WebSocket client. CI pipeline runs tests on every PR.
  - *Components:* `frontend`: vitest suite, test-helpers.ts, CI frontend-test job
  - *Done:* Vitest test suite with CI integration.
  - *Exempt:* Meta-requirement; test suite existence is the implementation

- [x] **REQ-018: Quick start guide and run script**
  - *Milestone:* M1
  - *Context:* New users need a beginner-friendly way to launch the full stack without reading the full README.
  - *Components:* `docs/quick-start.md`, `run.sh`
  - *Done:* `docs/quick-start.md` with platform-specific install steps and troubleshooting; `run.sh` launches backend + frontend in parallel with signal handling.
  - *Exempt:* Documentation only

- [x] **REQ-019: Model guide documentation**
  - *Milestone:* M1
  - *Context:* Users need to understand World 3 sectors, feedback loops, and how to interpret charts before exploring scenarios.
  - *Components:* `docs/model-guide.md`
  - *Done:* `docs/model-guide.md` with beginner + technical tracks covering all 5 sectors, 6 feedback loops, solver explanation, and preset comparison.
  - *Exempt:* Documentation only

- [x] **REQ-020: Variable descriptions content module**
  - *Milestone:* M1
  - *Context:* All UI text describing variables, parameters, and feedback loops must come from a single source of truth.
  - *Components:* `frontend/src/lib/content/variable-descriptions.ts`, ParameterInfoPanel, ParameterSlider info icon
  - *Done:* `frontend/src/lib/content/variable-descriptions.ts` — 16 variables, 12 parameters, 6 feedback loops with beginner + expert descriptions.

- [x] **REQ-021: Rich chart tooltips**
  - *Milestone:* M1
  - *Context:* Hovering on charts should show year, values per scenario, trend indicators, and a short variable description.
  - *Components:* `frontend`: TimeSeriesChart tooltip overlay
  - *Done:* Tooltip overlay in `TimeSeriesChart.svelte` with vertical guide line, per-scenario values, trend arrows, and description hint.

- [x] **REQ-022: Variable info panels**
  - *Milestone:* M1
  - *Context:* Users need on-demand deep explanations of any variable — beginner and expert level — accessible directly from charts.
  - *Components:* `frontend`: VariableInfoPanel, InfoPanelShell
  - *Done:* `VariableInfoPanel.svelte` slide-in panel triggered by chart title click; shows sector, unit, descriptions, feedback loops, and related variables.

- [x] **REQ-023: Simulation controls**
  - *Milestone:* M1
  - *Context:* Users should be able to adjust simulation time range and resolution without editing code.
  - *Components:* `frontend`: SimulationControls component, WS debounce
  - *Done:* `SimulationControls.svelte` with start year, end year, and time step dropdowns; 200ms debounce on WebSocket updates.

- [x] **REQ-024: Chart annotations**
  - *Milestone:* M1
  - *Context:* Charts should highlight key events (historical markers, peaks, threshold crossings) to guide interpretation.
  - *Components:* `frontend/src/lib/content/chart-annotations.ts`, D3 annotation rendering
  - *Done:* `frontend/src/lib/content/chart-annotations.ts` with static + dynamic annotations; rendered as dashed vertical lines with rotated labels in D3.

- [x] **REQ-025: Preset comparison UX**
  - *Milestone:* M1
  - *Context:* Users should be able to overlay multiple scenarios on the same charts and toggle visibility.
  - *Components:* `frontend`: ScenarioBar, ScenarioSelector
  - *Done:* `ScenarioBar` with click-to-toggle and double-click-to-focus; `ScenarioSelector` with "Compare All" button and per-scenario descriptions.

- [x] **REQ-026: Collapse historical calibration regression tests**
  - *Milestone:* M2
  - *Context:* The Collapse simulation must track real-world historical data within RMSE% thresholds over ~1960-2023. Quantitative comparison against World Bank, FAO, and OWID data exposes calibration gaps.
  - *Components:* `world3-cli`: `tests/historical_calibration.rs`; `data/historical/*.csv`
  - *RMSE% thresholds:* Population (<11%), Food/capita (<21%), IOPC (<19%), NNR fraction (<4%), Life expectancy (<12%)
  - *Max-year-error thresholds:* Population (<15%), Food/capita (<28%), IOPC (<38%), NNR fraction (<6%), Life expectancy (<19%)
  - *Current metrics:* Pop RMSE=8.1%, Food RMSE=18.6%, IOPC RMSE=16.2%, NNR RMSE=0.9%, LE RMSE=9.4%. Max-year: Pop 11.3%, Food 25.5%, IOPC 35.4%, NNR 2.6%, LE 15.6%.
  - *Model changes:* ISOPC dynamic service demand reference (lookup table replaces hardcoded 200.0), technology_growth_rate=0.014 (was 0.002, compensates for real-world TFP ~1.5%/yr), resource_efficiency=1.05, FIOAC consumption fraction capped at 0.70 (was 0.83), Delay3 for perceived life expectancy and pollution appearance (replacing Delay1), DCFS table calibrated for our model structure, HSAPC table + LMHS2 + EHSPC 20-year smooth for health services allocation, CMI(IOPC)×FPU(POP) crowding multiplier (replaces single crowding lookup), alic1=13yr (was 14yr).
  - *Structural limitations:* Population 1960s overshoot (31.7% max error -- model birth/death rates lag real demographic transition), IOPC post-2010 collapse (35.9% max error -- Collapse overshoot-collapse starts earlier than real world).
  - *Done:* All 5 RMSE + 5 max-year-error thresholds pass. 10 tests run in CI without `#[ignore]`.

- [x] **REQ-027: Bi-directional traceability**
  - *Milestone:* M1
  - *Context:* Coverage visibility and impact analysis require bidirectional links between requirements and tests.
  - *Components:* `scripts/traceability.py`, `docs/traceability-matrix.md`, CI traceability job
  - *Done:* `// REQ:` annotations in test files, CI script checks coverage, auto-generated traceability matrix.
  - *Exempt:* Infrastructure tooling; the traceability script itself is the implementation

- [x] **REQ-028: User feedback mechanism**
  - *Milestone:* M1
  - *Context:* Users need a way to report bugs and request features from within the app. A sidebar footer with GitHub Issues links provides a lightweight, zero-backend feedback channel.
  - *Components:* `frontend/src/components/Sidebar.svelte`, `frontend/src/lib/utils/feedback-url.ts`, `.github/ISSUE_TEMPLATE/`
  - *Done:* Sidebar footer with "Report a bug" and "Request a feature" links that open pre-filled GitHub Issues. Includes bug report and feature request issue templates.
  - *Exempt:* UI-only feature with unit tests on URL builder; no backend logic to test.

- [x] **REQ-037: Chart zoom and mobile touch UX**
  - *Milestone:* M1
  - *Context:* Mobile charts were unreadable (6 variables in ~300px, no zoom, broken touch). Desktop also benefits from zoom to inspect specific time periods.
  - *Components:* `frontend/src/lib/charts/UnifiedChart.svelte`, `frontend/src/lib/charts/zoom-helpers.ts`
  - *Done:* Replaced no-op `d3.brushX` with `d3.zoom()`. X-axis zoom/pan on all devices (wheel, pinch, drag). SVG clip path prevents overflow. Tap-to-pin tooltip on mobile (pointer: coarse). Hover tooltip on desktop. Reset zoom button + double-click/tap + Escape key. Y-axis auto-fits visible window in compare mode. Extracted pure zoom helpers with 30 unit tests.

- [x] **REQ-037: CLI — CI/CD validation**
  - *Milestone:* M1
  - *Context:* Automated pipelines need headless qualitative checks against Meadows 1972 reference dynamics. The CLI `validate` command runs without a browser or frontend.
  - *Components:* `world3-cli`: `validate` subcommand
  - *Done:* `world3-cli validate` checks population peak, resource depletion, pollution, industrial collapse, and life expectancy trajectories.
  - *Exempt:* Organizational requirement; CLI validate tested by REQ-005 tests

- [x] **REQ-038: CLI — Batch export**
  - *Milestone:* M1
  - *Context:* Researchers and external tools need raw simulation data in CSV format for analysis outside the webapp.
  - *Components:* `world3-cli`: `simulate --output` subcommand
  - *Done:* `world3-cli simulate --output <file>` exports 25-column CSV covering all World 3 stocks and derived variables.
  - *Exempt:* Organizational requirement; CSV output tested by REQ-003 tests

- [x] **REQ-039: CLI — Simulation debugging**
  - *Milestone:* M1
  - *Context:* Developers need structured analysis of simulation trajectories (peaks, phases, anomalies, oscillation detection) without starting a browser. Text/JSON output is more actionable than visual chart inspection for diagnosing model issues.
  - *Components:* `world3-cli`: `diagnose` subcommand
  - *Done:* `world3-cli diagnose` produces structured text/JSON reports with peak detection, phase analysis, anomaly flags, preset comparison, and dt-sensitivity stability checks.
  - *Exempt:* Organizational requirement; diagnose tested via agent workflow tests

- [x] **REQ-040: CLI — Reproducibility**
  - *Milestone:* M1
  - *Context:* Deterministic simulation runs from a single command ensure reproducible results across environments. Named presets provide canonical parameter sets.
  - *Components:* `world3-cli`: `simulate`, `presets` subcommands
  - *Done:* `world3-cli simulate --preset <name>` runs deterministic simulations; `world3-cli presets` lists available named parameter sets (Collapse, Technology, Stabilized).
  - *Exempt:* Organizational requirement; preset simulation tested by REQ-004/REQ-005 tests

- [x] **REQ-042: Sidebar-centric layout principle**
  - *Milestone:* M2
  - *Context:* The chart area is the primary display and should be maximized. All controls that change what the chart shows (scenario selection, parameter sliders, simulation settings, compare mode) belong in the sidebar. Info panels on the right provide context. On mobile, the sidebar becomes a slide-out drawer — all controls follow this pattern. The "Compare scenarios" toggle was previously in a dedicated toolbar row above the charts, wasting vertical space for a secondary feature. Moved to the sidebar, inline with the "Presets" header.
  - *Components:* `frontend/src/components/ScenarioSelector.svelte`, `frontend/src/components/ChartGrid.svelte`
  - *Done:* Compare toggle moved from ChartGrid toolbar to ScenarioSelector sidebar. ChartGrid toolbar removed.
  - *Exempt:* Layout/positioning concern; not testable with jsdom

- [x] **REQ-041: Click/tap chart lines to open info panel**
  - *Milestone:* M1
  - *Context:* Chart lines are not clickable — the only way to open a variable's info panel is via the legend. On desktop, clicking a line should open its info panel. On mobile, tapping a line opens the info panel; tapping the background shows the tooltip.
  - *Components:* `frontend/src/lib/charts/UnifiedChart.svelte`
  - *Done:* Invisible 20px-wide hit-area SVG paths on top of chart lines. Desktop click opens info panel with hover feedback. Mobile tap on line opens panel, tap on background shows tooltip. Hit-lines render above overlay with individual clip-path, handlers refreshed on D3 update selection.
  - *Exempt:* D3 SVG interaction; not testable with jsdom (no SVG pointer events)

---

## Abandoned

- [ ] **REQ-006: PNG chart output**
  - *Context:* Visual output of simulation results is needed for documentation and quick inspection.
  - *Abandoned:* Superseded by the interactive D3 frontend (REQ-009). The `--chart` flag and `plotters` dependency have been removed. The `diagnose` command (REQ-039) provides superior text-based analysis for debugging.
  - *Exempt:* Abandoned requirement; code and tests removed

- [ ] **REQ-011: Extension sectors — climate, energy, biodiversity, inequality**
  - *Context:* Originally a monolithic requirement for all 4 extension sectors. Split into scenario-specific requirements.
  - *Replaced by:* REQ-033 (climate + energy, M3) and REQ-034 (biodiversity + inequality, M4)

- [ ] **REQ-030: Multi-scenario historical calibration**
  - *Context:* Originally a monolithic requirement for calibrating all non-Collapse scenarios. Split into per-scenario requirements.
  - *Replaced by:* REQ-035 (Technology calibration, M3) and REQ-036 (Stabilized calibration, M4)

---

## In Progress

- [ ] **REQ-031: Scenario trajectory validation**
  - *Milestone:* M2
  - *Context:* Non-Collapse scenarios must produce meaningfully divergent futures compared to Collapse. Technology should show extended resource availability and delayed decline; Stabilized should show population leveling off and sustained output. Validates that preset parameter differences actually produce distinct trajectories.
  - *Components:* `world3-cli`: validate subcommand or dedicated test
  - *Priority:* high

---

## Planned

- [ ] **REQ-033: Climate and energy extension sectors**
  - *Milestone:* M3
  - *Context:* Adds CO2/EBM temperature model and energy mix sector to `world3-core`. Required for the Technotopia scenario to model technology-driven futures with climate and energy dynamics.
  - *Components:* `world3-core`: climate sector, energy sector
  - *Priority:* high

- [ ] **REQ-034: Biodiversity and inequality extension sectors**
  - *Milestone:* M4
  - *Context:* Adds LPI biodiversity index and Gini/HDI inequality model to `world3-core`. Required for the Ecotopia scenario to model justice and moderation dynamics.
  - *Components:* `world3-core`: biodiversity sector, inequality sector
  - *Priority:* high

- [ ] **REQ-035: Technology scenario historical calibration**
  - *Milestone:* M3
  - *Context:* Technology (Technotopia) preset must track real-world historical data over 1960-2023. Shared history with Collapse before policy switch years, then divergent future trajectory.
  - *Components:* `world3-cli`: `tests/historical_calibration.rs`; `data/historical/*.csv`
  - *Priority:* high

- [ ] **REQ-036: Stabilized scenario historical calibration**
  - *Milestone:* M4
  - *Context:* Stabilized (Ecotopia) preset must track real-world historical data over 1960-2023. Shared history with Collapse before policy switch years, then divergent future trajectory.
  - *Components:* `world3-cli`: `tests/historical_calibration.rs`; `data/historical/*.csv`
  - *Priority:* high

- [ ] **REQ-013: Live data ingestion pipeline**
  - *Milestone:* M5
  - *Context:* `world3-ingestion` crate with the `DataSource` trait, fetching live data from 7 external APIs with SQLite cache and fallback to bundled CSVs.
  - *Priority:* medium

- [ ] **REQ-014: Data mapping layer**
  - *Milestone:* M5
  - *Context:* A `mapping.rs` module to translate raw external observations into `WorldState` initial conditions, serving as the single source of truth for real-world-to-model conversion.
  - *Priority:* medium

- [ ] **REQ-015: Performance benchmarks and sensitivity analysis**
  - *Milestone:* M6
  - *Context:* Benchmarks for solver performance and sensitivity analysis to quantify how parameter changes affect model outputs.
  - *Priority:* medium
