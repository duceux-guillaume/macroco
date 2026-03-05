# CLI Cleanup Step 1: Remove Chart + Rework Requirements

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the dead `--chart` / plotters feature from the CLI, replace the overly broad REQ-003 with four focused CLI requirements (REQ-033..036), abandon REQ-006, and update all documentation.

**Architecture:** The CLI keeps `simulate`, `validate`, `diagnose`, `presets` subcommands. The only code change is removing the `--chart` flag and `render_chart()` function, plus dropping the `plotters` dependency. Everything else is documentation.

**Tech Stack:** Rust (Cargo.toml), Markdown docs

**Design:** `docs/plans/2026-03-05-cli-cleanup-design.md`

---

### Task 1: Remove plotters dependency and chart code from CLI

**Files:**
- Modify: `crates/world3-cli/Cargo.toml`
- Modify: `crates/world3-cli/src/main.rs`
- Modify: `Cargo.toml` (workspace root)

**Step 1: Remove `plotters` from workspace Cargo.toml**

In `Cargo.toml` (workspace root), delete lines 52-53:

```toml
# Charting
plotters = "0.3"
```

**Step 2: Remove `plotters` from CLI Cargo.toml**

In `crates/world3-cli/Cargo.toml`, delete line 20:

```toml
plotters = { workspace = true }
```

**Step 3: Remove chart code from main.rs**

In `crates/world3-cli/src/main.rs`:

1. Delete the `use plotters::prelude::*;` import (line 5)
2. Delete the `--chart` field from `Commands::Simulate` (lines 49-50):
   ```rust
           /// Output chart image (PNG) file path
           #[arg(long)]
           chart: Option<PathBuf>,
   ```
3. Update the destructure on line 95 — remove `chart` from the pattern:
   ```rust
   // Before:
   Commands::Simulate { preset, output, start, end, dt, chart } => {
   // After:
   Commands::Simulate { preset, output, start, end, dt } => {
   ```
4. Delete the chart rendering block (lines 124-127):
   ```rust
               if let Some(chart_path) = chart {
                   render_chart(&sim, &chart_path)?;
                   eprintln!("Wrote chart {}", chart_path.display());
               }
   ```
5. Delete the entire `render_chart()` function (lines 272-352)

**Step 4: Build to verify**

Run: `cargo build -p world3-cli`
Expected: compiles with no errors

**Step 5: Run workspace tests**

Run: `cargo test --workspace`
Expected: all tests pass

**Step 6: Commit**

```bash
git add -A
git commit -m "refactor: remove chart/plotters from CLI (REQ-006 abandoned)"
```

---

### Task 2: Clean up Dockerfile plotters dependencies

**Files:**
- Modify: `Dockerfile`

**Step 1: Remove plotters system deps from builder stage**

In `Dockerfile`, replace lines 6-9:

```dockerfile
# Install system deps for plotters (fontconfig)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
```

With:

```dockerfile
# Install system deps for Rust build
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
```

Note: `pkg-config` may still be needed for other native deps. `libssl-dev` is commonly needed for reqwest/TLS.

**Step 2: Remove fontconfig from runtime stage**

In `Dockerfile`, replace lines 50-52:

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
```

With:

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
```

**Step 3: Verify Docker build (optional — skip if no Docker available)**

Run: `docker build -t macroco-test .`
Expected: builds successfully

**Step 4: Commit**

```bash
git add Dockerfile
git commit -m "chore: remove plotters deps from Dockerfile"
```

---

### Task 3: Delete chart documentation and example files

**Files:**
- Delete: `docs/chart-output.md`
- Delete: `docs/examples/bau_standard_run.png`
- Delete: `docs/examples/` (directory, if empty after PNG removal)

**Step 1: Delete the files**

```bash
rm docs/chart-output.md
rm -r docs/examples/
```

**Step 2: Commit**

```bash
git add -A
git commit -m "docs: remove chart-output.md and example PNGs"
```

---

### Task 4: Update product-requirements.md

**Files:**
- Modify: `docs/product-requirements.md`

**Step 1: Mark REQ-003 as superseded**

Replace the REQ-003 entry (lines 19-22):

```markdown
- [x] **REQ-003: CLI batch simulation**
  - *Context:* Users need a command-line tool to run simulations, validate against reference trajectories, and export results.
  - *Components:* `world3-cli`: simulate, validate, presets subcommands
  - *Done:* `world3-cli` crate with `simulate`, `validate`, and `presets` subcommands; CSV output support.
```

With:

```markdown
- [x] **REQ-003: CLI batch simulation** *(Superseded by REQ-033..036)*
  - *Context:* Users need a command-line tool to run simulations, validate against reference trajectories, and export results.
  - *Components:* `world3-cli`
  - *Superseded:* Split into four focused requirements: REQ-033 (CI/CD validation), REQ-034 (batch export), REQ-035 (simulation debugging), REQ-036 (reproducibility).
```

**Step 2: Mark REQ-006 as abandoned**

Replace the REQ-006 entry (lines 34-37):

```markdown
- [x] **REQ-006: PNG chart output**
  - *Context:* Visual output of simulation results is needed for documentation and quick inspection.
  - *Components:* `world3-cli`: `--chart` flag, plotters crate
  - *Done:* `--chart` flag on `world3-cli simulate` renders multi-panel PNG via the `plotters` crate.
```

With:

```markdown
- [ ] **REQ-006: PNG chart output** *(Abandoned)*
  - *Context:* Visual output of simulation results is needed for documentation and quick inspection.
  - *Abandoned:* Superseded by the interactive D3 frontend (REQ-009). The `--chart` flag and `plotters` dependency have been removed. The `diagnose` command (REQ-035) provides superior text-based analysis for debugging.
```

**Step 3: Add REQ-033 through REQ-036 at the end of the Done section**

Insert before the `---` line that precedes "## In Progress" (before line 149):

```markdown
- [x] **REQ-033: CLI — CI/CD validation**
  - *Context:* Automated pipelines need headless qualitative checks against Meadows 1972 reference dynamics. The CLI `validate` command runs without a browser or frontend.
  - *Components:* `world3-cli`: `validate` subcommand
  - *Done:* `world3-cli validate` checks population peak, resource depletion, pollution, industrial collapse, and life expectancy trajectories.

- [x] **REQ-034: CLI — Batch export**
  - *Context:* Researchers and external tools need raw simulation data in CSV format for analysis outside the webapp.
  - *Components:* `world3-cli`: `simulate --output` subcommand
  - *Done:* `world3-cli simulate --output <file>` exports 22-column CSV covering all World 3 stocks and derived variables.

- [x] **REQ-035: CLI — Simulation debugging**
  - *Context:* Developers need structured analysis of simulation trajectories (peaks, phases, anomalies, oscillation detection) without starting a browser. Text/JSON output is more actionable than visual chart inspection for diagnosing model issues.
  - *Components:* `world3-cli`: `diagnose` subcommand
  - *Done:* `world3-cli diagnose` produces structured text/JSON reports with peak detection, phase analysis, anomaly flags, preset comparison, and dt-sensitivity stability checks.

- [x] **REQ-036: CLI — Reproducibility**
  - *Context:* Deterministic simulation runs from a single command ensure reproducible results across environments. Named presets provide canonical parameter sets.
  - *Components:* `world3-cli`: `simulate`, `presets` subcommands
  - *Done:* `world3-cli simulate --preset <name>` runs deterministic simulations; `world3-cli presets` lists available named parameter sets (BAU, Technology, Stabilized).

```

**Step 4: Update REQ-005 components to reference world3-core (preparation for Step 2)**

Replace the REQ-005 entry:

```markdown
- [x] **REQ-005: Validation against Meadows 1972**
  - *Context:* The standard BAU run must reproduce the qualitative dynamics of Meadows 1972 Fig. 35.
  - *Components:* `world3-cli`: validate subcommand
  - *Done:* `world3-cli validate` checks population peak, resource depletion, food and industrial output trajectories.
```

With:

```markdown
- [x] **REQ-005: Validation against Meadows 1972**
  - *Context:* The standard BAU run must reproduce the qualitative dynamics of Meadows 1972 Fig. 35.
  - *Components:* `world3-cli`: `validate` subcommand (REQ-033)
  - *Done:* `world3-cli validate` checks population peak, resource depletion, food and industrial output trajectories.
```

**Step 5: Commit**

```bash
git add docs/product-requirements.md
git commit -m "docs: supersede REQ-003, abandon REQ-006, add REQ-033..036"
```

---

### Task 5: Update architecture.md

**Files:**
- Modify: `docs/architecture.md`

**Step 1: Update CLI section**

Replace the CLI row in the Component Map table (line 15):

```markdown
| CLI | `crates/world3-cli/` | Batch simulation, validation, PNG charts, historical calibration tests | REQ-003, REQ-005, REQ-006, REQ-026 |
```

With:

```markdown
| CLI | `crates/world3-cli/` | Batch simulation, validation, debugging diagnostics, historical calibration tests | REQ-033, REQ-034, REQ-035, REQ-036, REQ-026 |
```

**Step 2: Update the CLI section body**

Replace lines 85-103 (the `## CLI` section through `### Historical Calibration Tests`):

```markdown
## CLI (`world3-cli`)

Implements: REQ-003, REQ-005, REQ-006, REQ-026

- Subcommands: `simulate`, `validate`, `diagnose`, `presets`.
- CSV output with 22 columns covering all World 3 stocks and derived variables.
- PNG chart rendering via the `plotters` crate.
- Validation compares BAU simulation output against Meadows 1972 reference trajectories.
- `diagnose` produces structured text/JSON reports for simulation debugging (peaks, phases, anomalies, oscillation detection, dt-sensitivity).
```

With:

```markdown
## CLI (`world3-cli`)

Implements: REQ-033, REQ-034, REQ-035, REQ-036, REQ-026

The CLI serves four roles:
1. **CI/CD validation** (REQ-033): `validate` runs headless qualitative checks against Meadows 1972.
2. **Batch export** (REQ-034): `simulate --output` exports 22-column CSV for external analysis.
3. **Simulation debugging** (REQ-035): `diagnose` produces structured text/JSON reports (peaks, phases, anomalies, oscillation detection, dt-sensitivity, preset comparison).
4. **Reproducibility** (REQ-036): `simulate --preset` and `presets` provide deterministic named runs.
```

**Step 3: Commit**

```bash
git add docs/architecture.md
git commit -m "docs: update architecture.md CLI section for new REQs"
```

---

### Task 6: Rewrite docs/cli.md

**Files:**
- Modify: `docs/cli.md`

**Step 1: Rewrite the full file**

Replace the entire content of `docs/cli.md` with:

```markdown
# CLI Reference

The `world3-cli` binary is the command-line interface for batch simulation, validation, debugging, and reproducibility.

## Purpose

The CLI complements the interactive web frontend for use cases that don't need a browser:

- **CI/CD validation** — headless qualitative checks in automated pipelines (REQ-033)
- **Batch export** — CSV output for external analysis tools like R, Python, or Excel (REQ-034)
- **Simulation debugging** — structured text/JSON analysis of trajectories, faster than visual inspection (REQ-035)
- **Reproducibility** — deterministic runs from a single command with named presets (REQ-036)

## Build & Install

` `` bash
cargo build --release --bin world3-cli

# Or run directly via cargo
cargo run --bin world3-cli -- <SUBCOMMAND> [OPTIONS]
` ``

## Commands

### `simulate`

Run a simulation and output results.

` ``bash
cargo run --bin world3-cli -- simulate [OPTIONS]
` ``

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `bau` | Scenario preset: `bau`, `technology`, `stabilized` |
| `--output <FILE>` | _(none)_ | Write results to CSV file |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |

**Output formats:**

- **No flags**: prints a summary table to stdout (every 10th year)
- **`--output`**: writes a 22-column CSV with all state variables

**Examples:**

` ``bash
# Summary table to stdout
cargo run --bin world3-cli -- simulate

# Full CSV output
cargo run --bin world3-cli -- simulate --preset bau --output output.csv

# Custom time range and step
cargo run --bin world3-cli -- simulate --preset stabilized \
  --start 1970 --end 2100 --dt 0.5 --output results.csv
` ``

**CSV columns (22 fields):**

` ``
year, population, cohort_0_14, cohort_15_44, cohort_45_64, cohort_65_plus,
birth_rate, death_rate, life_expectancy, fertility_rate,
industrial_capital, service_capital, industrial_output, industrial_output_per_capita,
service_output_per_capita, arable_land, food, food_per_capita, land_yield,
nnr_fraction, persistent_pollution, pollution_index
` ``

### `validate`

Validate the BAU run against Meadows 1972 reference checkpoints. Checks qualitative dynamics (not exact values):

1. Population at 1900, 1950, 1970 within expected ranges
2. Population peaks at 5B-12B between 1990-2080, then declines
3. NNR fraction monotonically decreasing, significantly depleted by 2100
4. Pollution peaks within expected range
5. IOPC peaks then collapses before 2100
6. Life expectancy peaks then falls

` ``bash
cargo run --bin world3-cli -- validate
` ``

Each checkpoint prints `PASS` or `FAIL`. The command exits with code 1 if any check fails.

### `diagnose`

Run structured simulation diagnostics for debugging model behavior.

` ``bash
cargo run --bin world3-cli -- diagnose [OPTIONS]
` ``

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `bau` | Scenario preset to analyze |
| `--compare <NAME>` | _(none)_ | Compare against a second preset |
| `--format <FMT>` | `text` | Output format: `text` or `json` |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |
| `--stability-check` | _(off)_ | Run dt-sensitivity analysis (dt, dt/2, dt/4) |

**Examples:**

` ``bash
# Text report for BAU
cargo run --bin world3-cli -- diagnose --preset bau

# Compare two scenarios
cargo run --bin world3-cli -- diagnose --preset bau --compare technology

# JSON output for scripting
cargo run --bin world3-cli -- diagnose --preset bau --format json

# Check numerical stability
cargo run --bin world3-cli -- diagnose --preset bau --stability-check
` ``

**Report sections:**
- **Peaks & troughs** — year and value of maxima/minima for each variable
- **Phases** — growth/decline periods with rates
- **Anomalies** — oscillation detection, rapid reversals
- **Stability** (with `--stability-check`) — per-variable convergence across dt halvings; flags UNSTABLE if drift >3%

### `presets`

List all available scenario presets.

` ``bash
cargo run --bin world3-cli -- presets
` ``

Output:

` ``
Available presets:
  bau          Business as Usual (original World 3 standard run)
  technology   Comprehensive Technology scenario
  stabilized   Stabilized World scenario
` ``

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (unknown preset, solver divergence, I/O failure, validation failure) |
```

Note: The triple backticks in the markdown above use a space between backticks for plan readability. When writing the actual file, use proper triple backticks without spaces.

**Step 2: Commit**

```bash
git add docs/cli.md
git commit -m "docs: rewrite CLI reference with purpose section and diagnose docs"
```

---

### Task 7: Update README.md, quick-start.md, and CLAUDE.md references

**Files:**
- Modify: `README.md`
- Modify: `docs/quick-start.md`
- Modify: `CLAUDE.md`

**Step 1: Remove chart-output.md link from README.md**

In `README.md`, delete line 29:

```markdown
| [Chart Output](docs/chart-output.md) | PNG chart rendering |
```

**Step 2: Remove chart image from quick-start.md**

In `docs/quick-start.md`, delete line 9:

```markdown
![BAU Standard Run](examples/bau_standard_run.png)
```

**Step 3: Update CLAUDE.md**

In `CLAUDE.md`, make these changes:

1. Remove `chart-output.md` from the Repository Structure listing. Delete line 59:
   ```
     chart-output.md          # PNG chart rendering feature
   ```

2. Remove `examples/` from the Repository Structure listing. Delete line 61:
   ```
     examples/                # Generated example charts
   ```

3. In the Commands section, remove the `--chart` reference. Replace:
   ```
   # Run simulation CLI
   cargo run --bin world3-cli -- simulate --preset bau --output output.csv
   ```
   (This line is fine — it doesn't mention `--chart`. No change needed.)

4. In the Debugging Workflow section, replace lines 191-192:
   ```
   - Prefer `diagnose` over `simulate --chart` when debugging model behavior — the text output contains all the information needed to reason about trajectory shape without reading a PNG.
   ```
   With:
   ```
   - Prefer `diagnose` over visual chart inspection when debugging model behavior — the text output contains all the information needed to reason about trajectory shape.
   ```

**Step 4: Commit**

```bash
git add README.md docs/quick-start.md CLAUDE.md
git commit -m "docs: remove chart references from README, quick-start, CLAUDE.md"
```

---

### Task 8: Regenerate traceability matrix and verify

**Files:**
- Regenerate: `docs/traceability-matrix.md`

**Step 1: Run traceability script**

Run: `python3 scripts/traceability.py`

Note: The script may report that new REQ-033..036 (marked Done) lack test coverage. This is expected — these are documentation/organizational requirements, not new code. Add `*Exempt:*` annotations to the REQs if the script fails.

**Step 2: If traceability fails, add exemptions**

In `docs/product-requirements.md`, add exemption lines to REQ-033..036:

```markdown
  - *Exempt:* Organizational requirement; CLI subcommands validated by existing REQ-005/REQ-026 tests
```

Then re-run: `python3 scripts/traceability.py`

**Step 3: Run full verification**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: all pass

Run: `cd frontend && npm run check && npm test`
Expected: all pass

**Step 4: Commit**

```bash
git add docs/traceability-matrix.md docs/product-requirements.md
git commit -m "docs: regenerate traceability matrix for CLI cleanup"
```

---

### Task 9: Final review and squash

**Step 1: Review all changes**

Run: `git log --oneline` to verify commit history.
Run: `git diff main --stat` to verify file changes match the plan.

Expected changed files:
- `Cargo.toml` — removed plotters workspace dep
- `crates/world3-cli/Cargo.toml` — removed plotters dep
- `crates/world3-cli/src/main.rs` — removed chart flag, render_chart(), plotters import
- `Dockerfile` — removed plotters system deps
- `docs/product-requirements.md` — REQ-003 superseded, REQ-006 abandoned, REQ-033..036 added
- `docs/architecture.md` — CLI section updated
- `docs/cli.md` — rewritten with purpose section and diagnose docs
- `docs/quick-start.md` — removed chart image
- `README.md` — removed chart-output link
- `CLAUDE.md` — removed chart references
- `docs/traceability-matrix.md` — regenerated

Expected deleted files:
- `docs/chart-output.md`
- `docs/examples/bau_standard_run.png`

**Step 2: Squash into logical commits (optional)**

If the user prefers a clean history, squash into 2-3 commits:
1. `refactor: remove chart/plotters from CLI and Dockerfile`
2. `docs: rework CLI requirements (REQ-033..036), abandon REQ-006`
3. `docs: rewrite CLI reference, update all doc references`
