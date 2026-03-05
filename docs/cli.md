# CLI Reference

The `world3-cli` binary is the command-line interface for batch simulation, validation, debugging, and reproducibility.

## Purpose

The CLI complements the interactive web frontend for use cases that don't need a browser:

- **CI/CD validation** — headless qualitative checks in automated pipelines (REQ-037)
- **Batch export** — CSV output for external analysis tools like R, Python, or Excel (REQ-038)
- **Simulation debugging** — structured text/JSON analysis of trajectories, faster than visual inspection (REQ-039)
- **Reproducibility** — deterministic runs from a single command with named presets (REQ-040)

## Build & Install

```bash
cargo build --release --bin world3-cli

# Or run directly via cargo
cargo run --bin world3-cli -- <SUBCOMMAND> [OPTIONS]
```

## Commands

### `simulate`

Run a simulation and output results.

```bash
cargo run --bin world3-cli -- simulate [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `collapse` | Scenario preset: `collapse`, `technology`, `stabilized` |
| `--output <FILE>` | _(none)_ | Write results to CSV file |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |

**Output formats:**

- **No flags**: prints a summary table to stdout (every 10th year)
- **`--output`**: writes a 25-column CSV with all state variables

**Examples:**

```bash
# Summary table to stdout
cargo run --bin world3-cli -- simulate

# Full CSV output
cargo run --bin world3-cli -- simulate --preset collapse --output output.csv

# Custom time range and step
cargo run --bin world3-cli -- simulate --preset stabilized \
  --start 1970 --end 2100 --dt 0.5 --output results.csv
```

**CSV columns (25 fields):**

```
year, population, cohort_0_14, cohort_15_44, cohort_45_64, cohort_65_plus,
birth_rate, death_rate, life_expectancy, fertility_rate,
industrial_capital, service_capital, industrial_output, industrial_output_per_capita,
service_output_per_capita, arable_land, potentially_arable_land, urban_industrial_land,
land_fertility, food, food_per_capita, land_yield,
nnr_fraction, persistent_pollution, pollution_index
```

### `validate`

Validate the Collapse run against Meadows 1972 reference checkpoints. Checks qualitative dynamics (not exact values):

1. Population at 1900, 1950, 1970 within expected ranges
2. Population peaks at 5B–16B between 2020–2090, then declines (widened for Delay3 model)
3. NNR fraction monotonically decreasing, significantly depleted by 2100
4. Pollution peaks within expected range
5. IOPC peaks then collapses to <50% of peak by 2100
6. Life expectancy peaks (45–80 yr) then declines to <80% of peak

```bash
cargo run --bin world3-cli -- validate
```

Each checkpoint prints `PASS` or `FAIL`. The command exits with code 1 if any check fails.

### `diagnose`

Run structured simulation diagnostics for debugging model behavior.

```bash
cargo run --bin world3-cli -- diagnose [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `collapse` | Scenario preset to analyze |
| `--compare <NAME>` | _(none)_ | Compare against a second preset |
| `--format <FMT>` | `text` | Output format: `text` or `json` |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |
| `--stability-check` | _(off)_ | Run dt-sensitivity analysis (dt, dt/2, dt/4) |

**Examples:**

```bash
# Text report for Collapse
cargo run --bin world3-cli -- diagnose --preset collapse

# Compare two scenarios
cargo run --bin world3-cli -- diagnose --preset collapse --compare technology

# JSON output for scripting
cargo run --bin world3-cli -- diagnose --preset collapse --format json

# Check numerical stability
cargo run --bin world3-cli -- diagnose --preset collapse --stability-check
```

**Report sections:**
- **Peaks & troughs** — year and value of maxima/minima for each variable
- **Phases** — growth/decline periods with rates
- **Anomalies** — oscillation detection, rapid reversals
- **Stability** (with `--stability-check`) — per-variable convergence across dt halvings; flags UNSTABLE if drift >3%

### `presets`

List all available scenario presets.

```bash
cargo run --bin world3-cli -- presets
```

Output:

```
Available presets:
  collapse     Collapse (original World 3 standard run)
  technology   Technotopia scenario
  stabilized   Stabilized World scenario
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (unknown preset, solver divergence, I/O failure, validation failure) |
