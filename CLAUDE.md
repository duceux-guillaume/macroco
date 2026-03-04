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

## Requirements & Architecture Traceability

- Product requirements with stable IDs: `docs/product-requirements.md`
- System architecture and component design: `docs/architecture.md`

**When to update these docs:**
- Adding a new feature → create a new REQ-NNN in product-requirements.md first
- Changing system design (new crate, new API endpoint, new store) → update architecture.md
- Completing a REQ → mark it done and verify Components field is accurate
- Writing a design doc in docs/plans/ → reference the REQ-NNN it addresses

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
  product-requirements.md  # Feature requirements (REQ-NNN IDs)
  architecture.md          # System design, components, data flow
  quick-start.md           # Beginner-friendly setup guide
  model-guide.md           # World 3 model explanation (beginner + expert tracks)
  simulation-engine.md     # World 3 model architecture, sectors, solver
  cli.md                   # CLI commands and flags reference
  api-server.md            # REST + WebSocket API documentation
  chart-output.md          # PNG chart rendering feature
  deployment.md            # Fly.io deployment guide
  examples/                # Generated example charts
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

# Validate against Meadows 1972 reference trajectories (BAU only — test Technology/Stabilized manually with simulate --preset)
cargo run --bin world3-cli -- validate

# Diagnose simulation output (structured text report)
cargo run --bin world3-cli -- diagnose --preset bau

# Compare two presets
cargo run --bin world3-cli -- diagnose --preset bau --compare technology

# JSON output for programmatic use
cargo run --bin world3-cli -- diagnose --preset bau --format json

# Check dt-sensitivity (runs at dt, dt/2, dt/4 and reports convergence)
cargo run --bin world3-cli -- diagnose --preset bau --stability-check

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

## Developer Conventions

> System architecture is in `docs/architecture.md`. Below are conventions and gotchas for working in this codebase.

### Permission & Autonomy Guidelines

Claude has broad tool permissions in this project. The deny-list in `.claude/settings.json` blocks dangerous shell commands. In addition, ALWAYS ask the user before:

- Deleting or renaming any file (even via Edit/Write — not just `rm`)
- Adding, removing, or upgrading dependencies (Cargo.toml, package.json)
- Any destructive git operation (force push, reset, rebase, branch delete)
- Modifying CI/CD configuration (.github/workflows/*)
- Changing .claude/settings.json permissions
- Running commands that send data to external services (deploy, publish, curl POST)

These actions require explicit user confirmation regardless of tool permissions.

Run `/permissions-audit` to review and improve permission settings.

### Simulation Engine (`world3-core`)
- Sector derivative order matters: resource_aux → capital → resource_depletion → agriculture → pollution → population. Documented in `derivatives.rs`.
- `WorldState::N` = 16 ODE stocks. When adding/removing stocks, update: `N`, `to_vec()`, `from_vec()`, `Add`/`Mul` impls, `derivatives.rs` (assembly + doc comments), and `initial_1900()`.
- `from_vec()` zeroes all auxiliary fields (non-ODE stocks like `food_per_capita`, `industrial_output`). Only ODE stocks (16 fields) survive RK4 intermediate stages (k2/k3/k4). For inter-sector feedback that must be consistent across solver stages, use ODE stocks (e.g., `food_per_capita_smooth`) not auxiliaries.
- `ScenarioParams::default()` must match BAU preset. When changing defaults, also update `data/presets/business_as_usual.json`.
- `LookupTable::eval()` clamps to endpoint y-values beyond the x-range (no extrapolation). When adding scenario params that push inputs beyond existing table ranges, extend the table.
- Our model omits World3-03's Land Fraction Harvested (LFH=0.7) and Processing Loss (PL=0.1), producing ~59% more food at identical parameters. Food-related tables (IFPC, FIOAA) are calibrated lower to compensate. BAU IOPC peaks at ~308.
- Lookup tables in `crates/world3-core/src/lookup/tables.rs` are audited against pyworld3 reference (World3-03 Vensim). See `docs/audit.md`. Run `/audit-tables` to re-audit after changes.
- pyworld3 reference: `https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json`
- Simulation is CPU-bound; always run via `tokio::task::spawn_blocking` to avoid blocking the async reactor.

### Frontend
- Historical CSV file stems in `data/historical/` MUST match IDs in `frontend/src/lib/charts/unified-config.ts`: `population`, `resources`, `food`, `industrial`, `pollution`, `life-expectancy`.
- Historical overlay uses combined min/max normalization (union of sim + historical data) so both lines fit [0,1] in UnifiedChart normalized mode.
- `frontend/src/lib/env.ts` provides `getApiBase()` / `getWsBase()` — returns relative URLs in production (empty `PUBLIC_*` vars), absolute URLs in dev. All API/WS imports use this, not `$env/static/public` directly.
- `frontend/.env.production` has empty `PUBLIC_API_BASE=` and `PUBLIC_WS_BASE=` to trigger same-origin fallback.
- Svelte 5 runes (`$state`, `$derived`, `$effect`) drive all reactivity. Use `$store` auto-subscription in `.svelte` files; use manual `.subscribe()` + `onDestroy(unsub)` only when you need side effects on store change (e.g. resetting local state).
- Info panels use composition: `InfoPanelShell` (shell chrome + Escape handler + expert toggle) → panel-specific content → shared `FeedbackLoops` + `RelatedVars` sub-components. Don't duplicate markup/CSS across panels.
- To style slotted child content from a parent component, use `:global()` scoped to a parent class: `.panel-body :global(section h3) { ... }`.
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
- Sector test modules use a shared `setup() -> (WorldState, ScenarioParams, WorldLookupTables)` that pre-populates upstream auxiliary fields. Reuse it; don't create new setup fns unless the file has none.
- Import `approx::assert_relative_eq` at module level in `#[cfg(test)] mod tests`, not inside individual test functions.
- `f64::parse()` accepts "NaN", "inf", "-inf" as valid. Always add `.is_finite()` guard when parsing external data destined for JSON serialization.
- `world3-cli` is a binary crate — use `cargo test -p world3-cli` (not `--lib`).
- Individual cohort derivatives can be negative even when total population grows (e.g. `d_cohort_0_14 < 0` at 1900 because aging-out exceeds births). Assert on net population, not individual cohorts.

### Debugging Workflow
- For simulation debugging, use `cargo run --bin world3-cli -- diagnose` instead of visual chart inspection.
- `diagnose --preset <name>` outputs a structured text report: peaks, troughs, phases, growth rates, anomalies.
- `diagnose --preset <name> --compare <other>` shows side-by-side deltas between two scenarios.
- `diagnose --format json` produces machine-readable output for programmatic assertions.
- Prefer `diagnose` over `simulate --chart` when debugging model behavior — the text output contains all the information needed to reason about trajectory shape without reading a PNG.
- When a user reports "the chart looks wrong", run `diagnose` first to identify which variable has unexpected peaks, phases, or anomalies, then investigate the relevant sector code.
- `diagnose` auto-detects oscillations (rapid alternating phase reversals) — check the Anomalies section for `Oscillation` entries.
- `diagnose --stability-check` runs at dt, dt/2, dt/4 and reports per-variable convergence. Use this when you suspect numerical instability (e.g., high phase counts, oscillating values). A variable drifting >1% between halvings is flagged UNSTABLE.
- After the IFPC food allocation rework, all presets (BAU, Technology, Stabilized) are stable at dt=1.0.

### Traceability
- Every test file/module must have a `// REQ: REQ-NNN` comment linking to the requirements it validates. Rust: place before `#[cfg(test)]`. TypeScript: first line of `.test.ts` file.
- Run `python3 scripts/traceability.py` locally to regenerate `docs/traceability-matrix.md`, then commit it. CI runs `--check` mode and fails if the matrix is stale.
- When adding a new `#[test]` or `describe()` block, include the `// REQ:` annotation.
- Docs-only and infrastructure requirements can be exempted with `- *Exempt:* <reason>` in `product-requirements.md`.
- Impact analysis: `grep -r '// REQ:.*REQ-005' crates/ frontend/` finds all tests covering a given requirement.

## Model Sectors (5 — original World 3)
Population · Industrial Capital · Agriculture · Non-Renewable Resources · Pollution

Future extensions (Milestone 2): Climate (CO₂/EBM temperature) · Energy Mix · Biodiversity (LPI) · Inequality (Gini/HDI)

## Environment Variables

```env
# Backend
RUST_LOG=info,world3_api=debug
STATIC_DIR=./static    # path to frontend build output (default: ./static)
HISTORICAL_DATA_DIR=./data/historical  # path to bundled historical CSVs (default: ./data/historical)
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

### Historical Calibration (REQ-026)
- BAU simulation must track real-world historical data within RMSE% thresholds over ~1960-2023.
- Variables: Population (<15%), Food/capita (<25%), IOPC (<30%), NNR fraction (<20%).
- Test: `cargo test -p world3-cli --test historical_calibration` (summary only; `-- --ignored` to run threshold checks)
- Design: `docs/plans/2026-03-04-bau-historical-calibration-design.md`
- Currently FAILING — thresholds are aspirational calibration targets.

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
