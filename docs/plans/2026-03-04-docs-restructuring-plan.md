# Documentation Restructuring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Consolidate requirements.md + 4 plan files into `docs/product-requirements.md` and `docs/architecture.md`, slim README.md to an index, and add traceability to CLAUDE.md.

**Architecture:** Pure documentation reorganization — no code changes. Content moves from root `requirements.md`, `docs/plans/*.md` design docs, `CLAUDE.md` architecture sections, and `README.md` into two new canonical docs. Old files deleted. CLAUDE.md and README.md trimmed.

**Tech Stack:** Markdown only.

---

### Task 1: Create `docs/product-requirements.md`

**Files:**
- Create: `docs/product-requirements.md`
- Read: `requirements.md` (source)
- Read: `docs/plans/2026-03-04-historical-data-overlay-design.md` (extract any missing REQ info)
- Read: `docs/plans/2026-03-04-parameter-explanation-design.md` (extract any missing REQ info)

**Step 1: Write the file**

Create `docs/product-requirements.md` with all REQs from `requirements.md`. Add a `Components:` field to each done REQ linking to architecture elements. Fix REQ-016 and REQ-017 which are marked `[x]` but appear under "Planned" — move them to "Done".

The traceability mapping for Components fields:

| REQ | Components |
|-----|------------|
| REQ-001 | `world3-core`: WorldState, sector ODEs, RK4 solver |
| REQ-002 | `world3-core`: LookupTable, `data/lookup_tables/` |
| REQ-003 | `world3-cli`: simulate, validate, presets subcommands |
| REQ-004 | `world3-core`: ScenarioParams, `data/presets/` |
| REQ-005 | `world3-cli`: validate subcommand |
| REQ-006 | `world3-cli`: `--chart` flag, plotters crate |
| REQ-007 | `world3-api`: Axum REST endpoints, scenario store |
| REQ-008 | `world3-api`: WebSocket handler, mpsc channel, `frontend` WS client |
| REQ-009 | `frontend`: SvelteKit app, D3 charts, parameter sliders, scenario bar |
| REQ-010 | `.github/workflows/`: clippy + test jobs |
| REQ-011 | _(planned — no components yet)_ |
| REQ-012 | `world3-api`: historical.rs, CSV parser, `/api/v1/historical` endpoint; `frontend`: historicalStore, chart overlays; `data/historical/*.csv` |
| REQ-013 | `world3-ingestion`: DataSource trait, SQLite cache _(planned)_ |
| REQ-014 | `world3-ingestion`: mapping.rs _(planned)_ |
| REQ-015 | _(planned — no components yet)_ |
| REQ-016 | Dockerfile, fly.toml, CI/CD deploy job, `world3-api` static serving |
| REQ-017 | `frontend`: vitest suite, test-helpers.ts, CI frontend-test job |
| REQ-018 | `docs/quick-start.md`, `run.sh` |
| REQ-019 | `docs/model-guide.md` |
| REQ-020 | `frontend/src/lib/content/variable-descriptions.ts`, ParameterInfoPanel, ParameterSlider info icon |
| REQ-021 | `frontend`: TimeSeriesChart tooltip overlay |
| REQ-022 | `frontend`: VariableInfoPanel, InfoPanelShell |
| REQ-023 | `frontend`: SimulationControls component, WS debounce |
| REQ-024 | `frontend/src/lib/content/chart-annotations.ts`, D3 annotation rendering |
| REQ-025 | `frontend`: ScenarioBar, ScenarioSelector |

Structure:
```
# Macroco — Product Requirements
> header with REQ-NNN explanation

## Done
  (all completed REQs with Context, Components, Done fields)

## In Progress
  (empty or current work)

## Planned
  (REQ-011, REQ-013, REQ-014, REQ-015 with Context, Priority)
```

**Step 2: Verify**

Manually check:
- All 25 REQ IDs present
- REQ-016 and REQ-017 under Done (not Planned)
- Every Done REQ has a Components field
- Planned REQs with no implementation have no Components field
- REQ-012 marked as Done (historical overlay is implemented)

**Step 3: Commit**

```bash
git add docs/product-requirements.md
git commit -m "docs: create product-requirements.md with REQ traceability"
```

---

### Task 2: Create `docs/architecture.md`

**Files:**
- Create: `docs/architecture.md`
- Read: `CLAUDE.md` lines 96–148 (Key Architecture Decisions)
- Read: `docs/plans/2026-03-04-historical-data-overlay-design.md` (architecture details)
- Read: `docs/plans/2026-03-04-parameter-explanation-design.md` (architecture details)

**Step 1: Write the file**

Create `docs/architecture.md` with this structure:

```markdown
# Macroco — System Architecture

## Overview
One paragraph: World 3 system dynamics model, Rust backend + SvelteKit frontend + D3 v7, deployed on Fly.io.

## Component Map
Table with columns: Component | Directory | Purpose | Implements

## Simulation Engine (`world3-core`)
Implements: REQ-001, REQ-002, REQ-004, REQ-005

Content from CLAUDE.md "Simulation Engine" section:
- WorldState typed struct, to_vec/from_vec at solver boundaries
- Sector derivative order
- WorldState::N = 16 ODE stocks, checklist for changes
- ScenarioParams::default() must match BAU preset
- LookupTable (piecewise-linear), audited against pyworld3
- spawn_blocking for CPU-bound work

## API Server (`world3-api`)
Implements: REQ-007, REQ-008, REQ-012

Content from CLAUDE.md "API Server" section:
- AppState composition
- Historical data: HashMap<String, HistoricalVariable>, loaded from CSV at startup
- Historical API: GET /api/v1/historical and /api/v1/historical/{variable_id}
- WebSocket: mpsc channel, 50ms debounce
- Static file serving with SPA fallback
- Graceful shutdown

## Frontend (`frontend`)
Implements: REQ-009, REQ-020, REQ-021, REQ-022, REQ-023, REQ-024, REQ-025

Content from CLAUDE.md "Frontend" section:
- env.ts: getApiBase/getWsBase
- Svelte 5 runes
- D3 direct rendering
- WS client auto-reconnect
- Content single source of truth (variable-descriptions.ts)
- Chart annotations (chart-annotations.ts)
- Info panels composition pattern (InfoPanelShell → content → sub-components)
- Style scoping with :global()

### Historical Data Overlay
Implements: REQ-012

From historical overlay design doc:
- CSV format with provenance headers in data/historical/
- Variable IDs must match unified-config.ts
- Combined min/max normalization for overlay
- Data flow: CSV → backend → API → frontend store → D3 dashed overlay

### Parameter Explanation Panels
Implements: REQ-020 (extension)

From parameter explanation design doc:
- Extended ParameterInfo interface (feedbackLoops, relatedVariables, impact)
- selectedParameterId / highlightedVariables stores (mutual exclusion with selectedVariableId)
- ParameterInfoPanel: 340px slide-in, sparkline, impact cards
- Chart highlighting: dim unrelated lines when parameter selected

## Data Pipeline (`world3-ingestion`)
Implements: REQ-013, REQ-014

Content from CLAUDE.md "Data Ingestion" section:
- DataSource trait, fetch → RawSourceData
- Fallback chain: live API → SQLite → bundled CSV
- mapping.rs as single source of truth

## CLI (`world3-cli`)
Implements: REQ-003, REQ-005, REQ-006

- Subcommands: simulate, validate, presets
- CSV output, PNG chart rendering (plotters crate)

## Deployment
Implements: REQ-016

- Multi-stage Dockerfile (Rust + Node + slim runtime)
- Fly.io: cdg region, auto-stop, graceful shutdown (15s drain / 20s kill)
- CI/CD: GitHub Actions, deploy on push to main

## CI/CD
Implements: REQ-010, REQ-017

- GitHub Actions: clippy → test → frontend-test → deploy
- Frontend tests: vitest + jsdom
- Ruleset: PR required, 1 approval, rebase-only, linear history
```

**Step 2: Verify**

Check:
- Every architecture section has `Implements: REQ-NNN` annotations
- All REQs from product-requirements.md are referenced at least once
- No developer conventions/gotchas in this file (those stay in CLAUDE.md)

**Step 3: Commit**

```bash
git add docs/architecture.md
git commit -m "docs: create architecture.md with REQ traceability"
```

---

### Task 3: Rewrite `README.md`

**Files:**
- Modify: `README.md` (full rewrite, ~40 lines)

**Step 1: Write the new README**

```markdown
# Macroco

Online live macroeconomic model based on the World 3 system dynamics model
(Meadows et al., *Limits to Growth*). Extended with modern indicators: climate,
energy mix, biodiversity, inequality.

## Quick Start

```bash
git clone <repo-url> && cd macroco
./run.sh            # builds + serves on http://localhost:5173
```

Requires Rust 1.75+ and Node.js 18+. See [Quick Start Guide](docs/quick-start.md) for full setup.

## Documentation

| Document | Description |
|----------|-------------|
| [Quick Start](docs/quick-start.md) | Install prerequisites and run locally |
| [Model Guide](docs/model-guide.md) | World 3 sectors, feedback loops, chart interpretation |
| [Product Requirements](docs/product-requirements.md) | Feature requirements with stable REQ-NNN IDs |
| [Architecture](docs/architecture.md) | System design, components, data flow |
| [Simulation Engine](docs/simulation-engine.md) | ODE model, sectors, RK4 solver |
| [CLI Reference](docs/cli.md) | Batch simulation commands and flags |
| [API Server](docs/api-server.md) | REST endpoints, WebSocket streaming |
| [Chart Output](docs/chart-output.md) | PNG chart rendering |
| [Deployment](docs/deployment.md) | Fly.io deployment guide |
| [Lookup Table Audit](docs/audit.md) | Audit against pyworld3 reference |

## License

GPL v3 — see [LICENSE](LICENSE).
```

**Step 2: Verify**

- ~35 lines
- All docs/ files linked
- No inline CLI/API docs (moved to respective doc files)

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: slim README to pitch + quick start + doc index"
```

---

### Task 4: Edit `CLAUDE.md`

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add traceability section**

Insert after the "Current Objective" section (after line 27), before "Repository Structure":

```markdown
## Requirements & Architecture Traceability

- Product requirements with stable IDs: `docs/product-requirements.md`
- System architecture and component design: `docs/architecture.md`

**When to update these docs:**
- Adding a new feature → create a new REQ-NNN in product-requirements.md first
- Changing system design (new crate, new API endpoint, new store) → update architecture.md
- Completing a REQ → mark it done and verify Components field is accurate
- Writing a design doc in docs/plans/ → reference the REQ-NNN it addresses
```

**Step 2: Trim architecture sections**

Replace the entire "Key Architecture Decisions" section (CLAUDE.md lines 96–148) with a short pointer:

```markdown
## Key Architecture Decisions

See `docs/architecture.md` for full system design with REQ traceability.

Key conventions for working in this codebase (not in architecture.md):
```

Then keep ONLY the developer conventions/gotchas that aren't system design — these are the "how to work in this codebase" items. Specifically keep:

**Simulation Engine conventions (keep):**
- Sector derivative order matters (developer gotcha)
- WorldState::N = 16 checklist for adding stocks
- ScenarioParams::default() must match BAU preset
- Lookup tables audited against pyworld3, run /audit-tables
- spawn_blocking for CPU-bound work

**API Server conventions (keep):**
- None beyond what's in architecture.md — remove this subsection header, fold any remaining items into a flat list

**Frontend conventions (keep):**
- Historical CSV file stems MUST match IDs in unified-config.ts
- Historical overlay normalization note
- env.ts: getApiBase/getWsBase usage rule
- .env.production empty vars rule
- Svelte 5 runes usage pattern
- Info panel composition pattern
- :global() scoping rule
- D3 direct rendering rationale
- WS client auto-reconnect
- Content single source of truth
- Chart annotations location

**Testing conventions (keep all — these are developer gotchas):**
- All Frontend Testing items
- All Backend Testing items

**Step 3: Update Repository Structure**

Update the repo structure section to include the new files and remove `requirements.md`:

```
docs/
  product-requirements.md  # Feature requirements (REQ-NNN IDs)
  architecture.md          # System design, components, data flow
  quick-start.md           # Beginner-friendly setup guide
  ...
```

Remove `plans/` from the structure listing since the old plans are deleted (new plans will still go there but it doesn't need to be in the static listing).

**Step 4: Verify**

- Traceability section present after Current Objective
- No system design prose in CLAUDE.md (all moved to architecture.md)
- All developer conventions/gotchas preserved
- Repository Structure updated

**Step 5: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add traceability section, move architecture to docs/architecture.md"
```

---

### Task 5: Delete old files

**Files:**
- Delete: `requirements.md`
- Delete: `docs/plans/2026-03-04-historical-data-overlay-design.md`
- Delete: `docs/plans/2026-03-04-historical-data-overlay-plan.md`
- Delete: `docs/plans/2026-03-04-parameter-explanation-design.md`
- Delete: `docs/plans/2026-03-04-parameter-explanation-plan.md`

**Step 1: Delete files**

```bash
git rm requirements.md
git rm docs/plans/2026-03-04-historical-data-overlay-design.md
git rm docs/plans/2026-03-04-historical-data-overlay-plan.md
git rm docs/plans/2026-03-04-parameter-explanation-design.md
git rm docs/plans/2026-03-04-parameter-explanation-plan.md
```

**Step 2: Verify**

```bash
# Ensure no broken references to deleted files
grep -r "requirements.md" docs/ CLAUDE.md README.md || echo "No references"
grep -r "historical-data-overlay" docs/ CLAUDE.md README.md || echo "No references"
grep -r "parameter-explanation" docs/ CLAUDE.md README.md || echo "No references"
```

Fix any remaining references.

**Step 3: Commit**

```bash
git commit -m "docs: remove requirements.md and completed plan files"
```

---

### Task 6: Final verification

**Step 1: Check all docs render and link correctly**

```bash
# All new files exist
ls docs/product-requirements.md docs/architecture.md

# No broken internal links (check markdown link targets exist)
grep -oP '\[.*?\]\((docs/[^)]+)\)' README.md | while read -r match; do
  file=$(echo "$match" | grep -oP 'docs/[^)]+')
  [ -f "$file" ] || echo "BROKEN: $file"
done

# requirements.md is gone
[ ! -f requirements.md ] && echo "OK: requirements.md deleted"

# Old plan files are gone
ls docs/plans/
# Should only show: 2026-03-04-docs-restructuring-design.md and this plan file
```

**Step 2: Verify REQ traceability is bidirectional**

Manually spot-check 3 REQs:
- REQ-001: appears in product-requirements.md with Components, referenced in architecture.md Simulation Engine
- REQ-012: appears in product-requirements.md with Components, referenced in architecture.md API Server + Historical Data Overlay
- REQ-020: appears in product-requirements.md with Components, referenced in architecture.md Frontend + Parameter Explanation

**Step 3: Verify CLAUDE.md has no architecture duplication**

Confirm CLAUDE.md doesn't describe system design (component relationships, data flow, API structure). Only developer conventions and gotchas remain.
