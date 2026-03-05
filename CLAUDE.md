# Macroco — Developer Guide

## Project Overview
Online live macroeconomic model based on World3-03 (Meadows et al., 2004 — *Limits to Growth: The 30-Year Update*), as implemented in [pyworld3](https://github.com/cvanwynsberghe/pyworld3). Extended with modern indicators: climate, energy mix, biodiversity, inequality.

**Stack:** Rust backend (Axum) + SvelteKit/TypeScript frontend + D3 v7. Deployed on Fly.io.

## Current Objective
Milestone 2: Collapse. The Collapse scenario must be historically calibrated against real-world data (1960-2023) and validated to reproduce the overshoot-and-collapse trajectory from Meadows 1972. REQ-031 (scenario trajectory validation) is the remaining in-progress work.

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
- Every REQ must have a `*Milestone:* MN` tag. Abandoned REQs go in the `## Abandoned` section with `*Replaced by:*` pointers.
- Changing system design (new crate, new API endpoint, new store) → update architecture.md
- Completing a REQ → mark it done and verify Components field is accurate

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
  presets/            # Named scenario parameter sets (Collapse, Technotopia, Ecotopia, LtG 1972).
docs/
  plans/                 # Working design/impl plans (gitignored — local only)
  product-requirements.md  # Feature requirements (REQ-NNN IDs)
  architecture.md          # System design, components, data flow
  quick-start.md           # Beginner-friendly setup guide
  model/                   # Per-variable model documentation (sectors, tables, parameters)
    README.md              # Index, deviation summary, chart guide
    sectors/               # One file per sector (5)
    tables/                # One file per lookup table (34)
    parameters/            # One file per scenario parameter (15)
    feedback-loops.md      # Cross-sector feedback loops
    solver.md              # RK4 solver, evaluation order
  simulation-engine.md     # World 3 model architecture, sectors, solver
  cli.md                   # CLI commands and flags reference
  api-server.md            # REST + WebSocket API documentation
  deployment.md            # Fly.io deployment guide
scripts/
  audit-model-doc.py       # CI-enforceable model doc sync checker
Dockerfile            # Multi-stage build (Rust + Node + slim runtime)
fly.toml              # Fly.io app configuration
.claude/commands/      # Project-local Claude Code slash commands (/audit-model-doc, /refine-pr)
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
cargo run --bin world3-cli -- simulate --preset collapse --output output.csv

# Validate qualitative dynamics (Collapse overshoot-and-collapse shape)
cargo test -p world3-cli --test qualitative_dynamics

# Diagnose simulation output (structured text report)
cargo run --bin world3-cli -- diagnose --preset collapse

# Compare two presets
cargo run --bin world3-cli -- diagnose --preset collapse --compare technology

# JSON output for programmatic use
cargo run --bin world3-cli -- diagnose --preset collapse --format json

# Check dt-sensitivity (runs at dt, dt/2, dt/4 and reports convergence)
cargo run --bin world3-cli -- diagnose --preset collapse --stability-check

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

- `docs/plans/` contains internal design/implementation plans. Exclude them from PRs (they're working documents, not deliverables).

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

### Pre-PR Workflow

Before creating or updating a PR, run `/refine-pr`. This executes phased quality gates:

1. **Validate** — tests, clippy, frontend checks, traceability (stops on failure)
2. **Refine** — `/simplify`, fold plan findings into permanent docs (modifies files)
3. **Review** — `/requesting-code-review`, `/permissions-audit`, `/claude-md-management:revise-claude-md` (advisory)

Plan files (`docs/plans/`) are gitignored. They are working documents for Claude sessions — valuable findings must be folded into CLAUDE.md, architecture.md, or product-requirements.md before the PR.

### Simulation Engine (`world3-core`)
- Sector derivative order matters: resource_aux → capital → resource_depletion → agriculture → pollution → population. Documented in `derivatives.rs`.
- `WorldState::N` = 21 ODE stocks. When adding/removing stocks, update: `N`, `to_vec()`, `from_vec()`, `Add`/`Mul` impls, `derivatives.rs` (assembly + doc comments), and `initial_1900()`. The 21 stocks include: 4 population cohorts, industrial/service capital, 3 agriculture stocks, NNR, persistent pollution, 2 smoothing stocks, 4 Delay3 pipeline stages (2 for perceived life expectancy, 2 for pollution appearance), and 1 EHSPC smooth stock (effective health services per capita, 20-year smooth).
- `from_vec()` zeroes all auxiliary fields (non-ODE stocks like `food_per_capita`, `industrial_output`). Only ODE stocks (21 fields) survive RK4 intermediate stages (k2/k3/k4). For inter-sector feedback that must be consistent across solver stages, use ODE stocks (e.g., `food_per_capita_smooth`) not auxiliaries.
- `ScenarioParams::default()` must match Collapse preset. When changing defaults, also update `data/presets/collapse.json`.
- `LookupTable::eval()` clamps to endpoint y-values beyond the x-range (no extrapolation). When adding scenario params that push inputs beyond existing table ranges, extend the table.
- Our model includes World3-03's Land Fraction Harvested (LFH=0.7) and Processing Loss (PL=0.1) in the food equation. Lookup tables are aligned to pyworld3 reference values with four intentional deviations: FIOACV is smoothed above IOPC=400 (pyworld3 has a cliff from 0.43→0.73 that traps IOPC), FIOAA has a 0.005 floor at high food_ratio (prevents oscillation in Ecotopia preset), FIOAC consumption fraction is capped at 0.70 (pyworld3 goes to 0.83, which over-allocates to consumption and suppresses IOPC growth), and DCFS (desired completed family size) is calibrated for our model structure rather than exact pyworld3 match.
- Perceived life expectancy uses a Delay3 (3-stage cascaded delay), matching World3-03 specification. Pollution appearance also uses Delay3. Both add 2 intermediate pipeline stages each to WorldState (4 extra ODE stocks total).
- ISOPC lookup table provides dynamic service demand reference based on IOPC (replaces hardcoded 200.0). This allows service allocation to scale with industrial development.
- Collapse `technology_growth_rate` = 0.014, `resource_efficiency` = 1.05 (compensates for real-world TFP growth ~1.5%/yr that the original 1972 model did not anticipate).
- Collapse `resource_efficiency_growth_rate` = 0.0035 (0.35%/yr from 1970; real-world extraction tech improvement). Technotopia uses 0.007 (2× Collapse — optimistic extraction tech). Ecotopia uses 0.0 (static `resource_efficiency=4.0` instead).
- Collapse `agricultural_technology_growth_rate` = 0.005 (Macroco extension: Green Revolution TFP from 1960; USDA ERS ~1%/yr; set to 0.005 because LYMC captures input-driven gains). Technotopia uses 0.006. Ecotopia uses 0.0 (static `agricultural_technology=2.0` instead).
- Collapse model parameter changes can cause bifurcations: e.g., tech_rate >0.002 shifts population peak from ~2030 to ~2073. Always run `cargo test -p world3-cli --test qualitative_dynamics` and `diagnose` after parameter changes to catch qualitative shifts.
- Lookup tables in `crates/world3-core/src/lookup/tables.rs` are audited against pyworld3 reference (World3-03 Vensim). Each table has a doc file in `docs/model/tables/`. Run `/audit-model-doc` to re-audit after changes. Modes: default (full audit + fix), `--check` (CI, read-only), `--diff` (pre-PR, changed files only).
- When modifying sector code, lookup tables, or parameters, update the corresponding file in `docs/model/`. The `/audit-model-doc --diff` gate in `/refine-pr` will catch missed updates.
- Parameter doc files (`docs/model/parameters/*.md`) must follow the template: H1 title, `**Sector:**`, `**Source code:**`, `**BAU value:**` metadata lines, then `## Equation Context`, `## Calibration`, `## Info Panel`, `## References` sections. The audit enforces this structure.
- pyworld3 reference: `https://github.com/cvanwynsberghe/pyworld3/blob/master/pyworld3/functions_table_world3.json`
- Simulation is CPU-bound; always run via `tokio::task::spawn_blocking` to avoid blocking the async reactor.
- `world3_core::validation::validate_collapse()` thresholds must stay aligned with `world3-core/tests/qualitative_dynamics.rs` bounds on main. After rebasing, check both if model parameters changed upstream.

### Frontend
- Layout principle: sidebar = controls, center = chart (primary display), right = info panels. All controls that change chart display belong in the sidebar. On mobile, sidebar becomes a drawer — same pattern for all controls.
- Svelte components live in `frontend/src/components/`, NOT `frontend/src/lib/components/`. Utilities/stores/types live in `frontend/src/lib/`.
- SSR guard: use `import { browser } from '$app/environment'`, not `typeof navigator !== 'undefined'`.
- Historical CSV file stems in `data/historical/` MUST match IDs in `frontend/src/lib/charts/unified-config.ts`: `population`, `resources`, `food`, `industrial`, `pollution`, `life-expectancy`.
- Historical overlay uses combined min/max normalization (union of sim + historical data) so both lines fit [0,1] in UnifiedChart normalized mode.
- `frontend/src/lib/env.ts` provides `getApiBase()` / `getWsBase()` — returns relative URLs in production (empty `PUBLIC_*` vars), absolute URLs in dev. All API/WS imports use this, not `$env/static/public` directly.
- `frontend/.env.production` has empty `PUBLIC_API_BASE=` and `PUBLIC_WS_BASE=` to trigger same-origin fallback.
- Svelte 5 runes (`$state`, `$derived`, `$effect`) drive all reactivity. Use `$store` auto-subscription in `.svelte` files; use manual `.subscribe()` + `onDestroy(unsub)` only when you need side effects on store change (e.g. resetting local state).
- For `<select>` elements bound to Svelte stores, prefer `bind:value={$store}` over manual `onchange` handlers.
- Info panels use composition: `InfoPanelShell` (shell chrome + Escape handler + expert toggle) → panel-specific content → shared `FeedbackLoops` + `RelatedVars` sub-components. Don't duplicate markup/CSS across panels.
- To style slotted child content from a parent component, use `:global()` scoped to a parent class: `.panel-body :global(section h3) { ... }`.
- D3 is used directly (not wrapped in a chart library) because World 3 output requires custom multi-axis, phase-plane, and animated transition patterns.
- Historical line IDs use `HIST_LINE_PREFIX` (`hist-`) prefix on the variable ID (e.g., `hist-population`). Strip prefix to map back to `unifiedVariables` config.
- Use `get()` from `svelte/store` for synchronous store reads outside reactive contexts, not manual subscribe/unsubscribe.
- WS client auto-reconnects with 2s backoff. All WS messages are typed against `WsClientMsg` / `WsServerMsg`.
- All variable/parameter descriptions live in `frontend/src/lib/content/variable-descriptions.ts` — single source of truth.
- `variable-descriptions.ts` is **generated** by `python3 scripts/generate-variable-descriptions.py` from `## Info Panel` sections in `docs/model/` files. Never edit it manually — edit the doc source and regenerate. Use `--check` to verify freshness.
- Chart annotations (peaks, thresholds) defined in `frontend/src/lib/content/chart-annotations.ts`.

### Frontend Testing
- Git worktrees don't share `frontend/node_modules`. Run `cd frontend && npm install` before frontend checks in a new worktree.
- Test stack: vitest + jsdom. Config in `vite.config.ts` (import `defineConfig` from `vitest/config`, not `vite`).
- Test helpers in `frontend/src/lib/test-helpers.ts` — `makeWorldState()` and `makeTimeSeries()` factories.
- Mock SvelteKit env: `vi.mock('$env/static/public', () => ({ PUBLIC_API_BASE: '...' }))` before imports.
- Svelte stores in tests: use `get()` from `svelte/store`; reset writable stores in `beforeEach`.

### Backend Testing
- `approx` crate `assert_relative_eq!` does NOT support format string messages — use separate `assert!` for custom messages.
- Run full verification after model changes: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- Sector test modules use a shared `setup() -> (WorldState, ScenarioParams, WorldLookupTables)` that pre-populates upstream auxiliary fields. Reuse it; don't create new setup fns unless the file has none.
- Import `approx::assert_relative_eq` at module level in `#[cfg(test)] mod tests`, not inside individual test functions.
- `f64::parse()` accepts "NaN", "inf", "-inf" as valid. Always add `.is_finite()` guard when parsing external data destined for JSON serialization.
- `world3-cli` is a binary crate — use `cargo test -p world3-cli` (not `--lib`).
- Individual cohort derivatives can be negative even when total population grows (e.g. `d_cohort_0_14 < 0` at 1900 because aging-out exceeds births). Assert on net population, not individual cohorts.
- `ScenarioMeta::default()` generates a random hex ID via `scenario_id()`. Preset scenarios in the store are keyed by this hash, not by human-readable names like `"collapse"`. Tests needing to run a simulation should use inline `params` rather than looking up by scenario ID.
- `init_app_state()` loads historical CSVs from `HISTORICAL_DATA_DIR` (default `./data/historical`). Integration tests using it must run from the repo root or set the env var.
- Historical calibration tests: `cargo test -p world3-core --test historical_calibration -- --nocapture` to see the summary report. Uses `OnceLock` to share one Collapse sim across all tests.
- Calibration test helpers (CSV loader, RMSE%, max-year-error, match_years) live in `crates/world3-core/tests/common/mod.rs`. Reuse them for new scenario calibration tests (e.g., Ecotopia REQ-036).

### Debugging Workflow
- For simulation debugging, use `cargo run --bin world3-cli -- diagnose` instead of visual chart inspection.
- `diagnose --preset <name>` outputs a structured text report: peaks, troughs, phases, growth rates, anomalies.
- `diagnose --preset <name> --compare <other>` shows side-by-side deltas between two scenarios.
- `diagnose --format json` produces machine-readable output for programmatic assertions.
- Prefer `diagnose` over visual chart inspection when debugging model behavior — the text output contains all the information needed to reason about trajectory shape.
- When a user reports "the chart looks wrong", run `diagnose` first to identify which variable has unexpected peaks, phases, or anomalies, then investigate the relevant sector code.
- `diagnose` auto-detects oscillations (rapid alternating phase reversals) — check the Anomalies section for `Oscillation` entries.
- `diagnose --stability-check` runs at dt, dt/2, dt/4 and reports per-variable convergence. Use this when you suspect numerical instability (e.g., high phase counts, oscillating values). A variable drifting >3% between halvings is flagged UNSTABLE. Pollution peak is the most dt-sensitive variable (~2.4% drift).
- After the IFPC food allocation rework, all presets (Collapse, Technotopia, Ecotopia) are stable at dt=1.0.

### Traceability
- Every test file/module must have a `// REQ: REQ-NNN` comment linking to the requirements it validates. Rust: place before `#[cfg(test)]`. TypeScript: first line of `.test.ts` file.
- Run `python3 scripts/traceability.py` locally to regenerate `docs/traceability-matrix.md`, then commit it. CI runs `--check` mode and fails if the matrix is stale. The script exits non-zero when any Done REQ lacks test coverage — this is a coverage gate, not a script error; the matrix is still written.
- The traceability script determines REQ status from section headers (`## Done`, `## Planned`, etc.), NOT from checkbox markers (`[x]`). Moving a REQ between sections is required to change its status.
- When adding a new `#[test]` or `describe()` block, include the `// REQ:` annotation.
- Docs-only and infrastructure requirements can be exempted with `- *Exempt:* <reason>` in `product-requirements.md`.
- Impact analysis: `grep -r '// REQ:.*REQ-005' crates/ frontend/` finds all tests covering a given requirement.
- When marking a REQ as Done in `product-requirements.md`, also run `python3 scripts/traceability.py` to update the matrix.

## Model Sectors (5 — original World 3)
Population · Industrial Capital · Agriculture · Non-Renewable Resources · Pollution

### Scenario Naming
- Collapse (preset: `collapse`) = **Collapse** — default trajectory, overshoot and decline
- Technology (preset: `technology`) = **Technotopia** — technology and resource discovery save the day
- Stabilized (preset: `stabilized`) = **Ecotopia** — humanity progresses toward justice and moderation
- Technotopia has NO social changes (family_planning_efficacy=0.0). Family planning belongs in Ecotopia only.

Future extensions: Climate + Energy (M3 Technotopia, REQ-033) · Biodiversity + Inequality (M4 Ecotopia, REQ-034)

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
The "standard run" (Collapse preset, 1900–2100, no policy interventions) must reproduce Meadows 1972 Fig. 35 dynamics:
- Global population peaks ~2030 at ~8B then declines
- Non-renewable resources fall to ~50% of initial by ~2050
- Food per capita peaks mid-century then falls
- Industrial output per capita peaks and collapses before 2100

Run `cargo test -p world3-cli --test qualitative_dynamics` to check Collapse overshoot-and-collapse trajectory shape.

### Historical Calibration (REQ-026)
- Collapse simulation must track real-world historical data within RMSE% and max-year-error thresholds over ~1960-2023.
- RMSE% thresholds: Population (<11%), Food/capita (<15%), IOPC (<16%), NNR fraction (<4%), Life expectancy (<12%).
- Max-year-error thresholds: Population (<15%), Food/capita (<20%), IOPC (<38%), NNR fraction (<7%), Life expectancy (<19%).
- Test: `cargo test -p world3-core --test historical_calibration` (11 tests: 5 RMSE + 5 max-year-error + 1 summary)
- All 10 thresholds PASS (Pop RMSE=7.9%, Food=7.7%, IOPC=14.9%, NNR=1.1%, LE=11.1%; Max-year: Pop 11.5%, Food 12.1%, IOPC 35.4%, NNR 6.0%, LE=15.6%).

### Technotopia Calibration (REQ-035)
- Technotopia uses approach B (param tuning only): `technology_growth_rate=0.017`, `resource_efficiency_growth_rate=0.007`, `pollution_control=0.15`, `ag_tech_growth_rate=0.006`.
- RMSE% thresholds: Population (<13%), Food/capita (<18%), IOPC (<22%), NNR fraction (<6%), Life expectancy (<14%).
- Max-year-error thresholds: Population (<18%), Food/capita (<25%), IOPC (<42%), NNR fraction (<15%), Life expectancy (<22%).
- Test: `cargo test -p world3-core --test technotopia_historical_calibration` (14 tests: 5 RMSE + 5 max-year-error + 3 qualitative + 1 summary)
- All thresholds PASS (Pop RMSE=8.6%, Food=5.5%, IOPC=16.2%, NNR=1.8%, LE=11.5%).
- Qualitative: population >6B at 2100, min IOPC >120 post-2050, food >230 at 2100.

## CI/CD
- GitHub Actions: clippy → test → frontend-test → deploy (on push to main only)
- PR preview deploy: add `deploy-preview` label to any PR → deploys to macroco.fly.dev; remove label / merge / close → auto-reverts to main
- Deploy gated on `environment: production` with required status checks
- `production` environment has branch policy: only `main` can deploy. PR deploys use `preview` environment (separate `FLY_API_TOKEN` secret).
- Workflow jobs needing `gh pr comment` require `permissions: pull-requests: write`.
- When a job uses `needs:` but upstream jobs may be skipped (not failed), use `always()` + `!contains(needs.*.result, 'failure')` in the `if:` condition.
- GitHub `unlabeled` event: `github.event.pull_request.labels` is post-removal. Check `github.event.label.name` for the removed label.
- `frontend-test` job runs: `npm run check`, `npm test`, `npm run build`
- Ruleset on main: PR required (1 approval), rebase-only, linear history, no force push
- `THIRD_PARTY_LICENSES` must be generated without `node_modules` present (CI doesn't install it). Temporarily rename `frontend/node_modules` before running `python3 scripts/generate-third-party-licenses.py`, then restore it.

## Product Milestones
1. **M1 — Foundation** (complete): Engine, UX, API, CLI, docs, CI/CD.
2. **M2 — Collapse** (current): Collapse historical calibration, trajectory validation.
3. **M3 — Technotopia**: Climate + energy sectors, Technology scenario calibration.
4. **M4 — Ecotopia**: Biodiversity + inequality sectors, Ecotopia scenario calibration.
5. **M5 — Living Data**: `world3-ingestion` crate, 7 data sources, SQLite cache.
6. **M6 — Deep Exploration**: Advanced charting, benchmarks, sensitivity analysis.

## License
AGPL-3.0-or-later — see LICENSE file. Run `python3 scripts/generate-third-party-licenses.py` to regenerate THIRD_PARTY_LICENSES.
