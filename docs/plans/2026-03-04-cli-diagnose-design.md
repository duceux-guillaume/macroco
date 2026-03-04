# CLI Diagnose Command — Design Document

**Date:** 2026-03-04
**Status:** Approved
**Goal:** Enable structured, text-based simulation diagnostics so Claude Code agents can debug model behavior without visual chart inspection.

## Problem

When debugging simulation issues, the current workflow requires visually inspecting PNG charts or frontend D3 charts. Claude Code cannot see images, so the user must manually describe what looks wrong — slow, lossy, and error-prone. The existing `validate` command only checks 14 pass/fail checkpoints and doesn't provide the rich trajectory analysis needed for debugging.

## Solution

A new `world3-cli diagnose` subcommand that runs a simulation and outputs a structured text report: peaks, troughs, phases, growth rates, inflection points, anomalies, and (optionally) comparative deltas between two presets. Also supports JSON output for programmatic consumption.

## CLI Interface

```
# Single-run diagnostics
world3-cli diagnose --preset bau
world3-cli diagnose --preset technology --format json

# Comparative diagnostics
world3-cli diagnose --preset bau --compare technology
world3-cli diagnose --preset bau --compare technology --format json

# Custom time range
world3-cli diagnose --preset bau --start 1900 --end 2100 --dt 1.0
```

Clap definition:

```rust
Diagnose {
    #[arg(long, default_value = "bau")]
    preset: String,

    #[arg(long)]
    compare: Option<String>,

    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    format: String,

    #[arg(long, default_value_t = 1900.0)]
    start: f64,

    #[arg(long, default_value_t = 2100.0)]
    end: f64,

    #[arg(long, default_value_t = 1.0)]
    dt: f64,
}
```

Output to stdout. Progress/status to stderr.

## Data Model

### Single-run analysis

```rust
struct SimDiagnostics {
    preset_name: String,
    time_range: (f64, f64),
    dt: f64,
    num_steps: usize,
    variables: Vec<VariableDiagnostics>,
    anomalies: Vec<Anomaly>,
}

struct VariableDiagnostics {
    name: String,
    unit: String,
    initial: f64,
    final_value: f64,
    peak: ValueAtYear,
    trough: ValueAtYear,
    phases: Vec<Phase>,
    inflection_points: Vec<ValueAtYear>,
    is_monotonic: bool,
    max_growth_rate: ValueAtYear,
    max_decline_rate: ValueAtYear,
}

struct Phase {
    kind: PhaseKind,  // Growing, Declining, Plateau
    start_year: f64,
    end_year: f64,
    start_value: f64,
    end_value: f64,
    avg_annual_rate: f64,
}

struct Anomaly {
    year: f64,
    variable: String,
    kind: AnomalyKind,  // Negative, NaN, Inf, Discontinuity
    value: f64,
}
```

### Comparison

```rust
struct ComparativeDiagnostics {
    baseline: SimDiagnostics,
    comparison: SimDiagnostics,
    deltas: Vec<VariableDelta>,
}

struct VariableDelta {
    name: String,
    peak_value_change: f64,
    peak_value_pct_change: f64,
    peak_year_shift: f64,
    final_value_change: f64,
    trajectory_changed: bool,
    phase_diff: String,
}
```

All structs derive `Serialize` for JSON output.

## Tracked Variables (6)

Population, Food/capita, Industrial output/capita, Services/capita, NNR fraction, Pollution index.

Same set as the existing chart and validate command.

## Text Output Format

### Single run

```
=== Simulation Diagnostics: Business as Usual ===
Time: 1900-2100, dt=1.0yr, 201 steps

-- Population ------------------------------------------------
  Initial (1900):  1.61e9
  Peak:            7.21e9  at year 2032
  Trough:          3.44e9  at year 2100
  Final (2100):    3.44e9
  Phases:          Growing 1900-2032 (+1.2%/yr avg) -> Declining 2032-2100 (-1.1%/yr avg)
  Max growth rate: +2.1%/yr at 1968
  Max decline rate: -2.3%/yr at 2058

-- Food / capita ---------------------------------------------
  ...

-- Anomalies -------------------------------------------------
  None detected.
```

### Comparison

Each variable section gets a delta block:

```
-- Population ------------------------------------------------
  Baseline (bau):     peak 7.21e9 at 2032, final 3.44e9
  Comparison (tech):  peak 9.08e9 at 2044, final 6.12e9
  D peak:  +1.87e9 (+26.0%), 12 years later
  D final: +2.68e9 (+77.9%)
  Trajectory: Growing phase extended by 12 years
```

## Module Structure

```
crates/world3-cli/src/
  main.rs              # add Diagnose variant to Commands enum
  diagnose/
    mod.rs             # run_diagnose() entry point
    analysis.rs        # SimDiagnostics computation — pure, no I/O
    format_text.rs     # fn format_text(SimDiagnostics) -> String
    format_json.rs     # fn format_json(SimDiagnostics) -> String
    compare.rs         # ComparativeDiagnostics, VariableDelta diff logic
```

No new crate dependencies. `serde`/`serde_json` already available.

## Testing Strategy

### 1. Unit tests (`analysis.rs`)

Test analysis functions with synthetic data:

- Peak detection on known series
- Monotonic series → single Declining phase
- Grow-then-decline → two phases with correct boundaries
- NaN/negative anomaly detection
- Plateau classification
- Inflection point detection on sigmoid-like curve

### 2. End-to-end regression tests (`diagnose/mod.rs`)

Run actual BAU simulation through `run_analysis()` and assert on known properties:

- Population peaks 2000–2070, value 5B–12B
- NNR is monotonically declining, single phase
- No anomalies in standard BAU
- Comparative: technology has later population peak than BAU
- All deltas computed without NaN

### 3. Agent workflow test (`tests/agent-workflow/test_diagnose_workflow.sh`)

Shell script proving the debugging workflow:

1. Single-run text output contains expected sections (Population, Peak, Phases)
2. JSON output is valid and queryable with `jq`
3. Comparison mode produces delta analysis
4. Agent can extract specific diagnostics (decline start year) from JSON without chart reading

## CLAUDE.md Update

Add under Key Architecture Decisions:

```markdown
### Debugging Workflow
- For simulation debugging, use `cargo run --bin world3-cli -- diagnose` instead of visual chart inspection.
- `diagnose --preset <name>` outputs a structured text report: peaks, troughs, phases, growth rates, anomalies.
- `diagnose --preset <name> --compare <other>` shows side-by-side deltas between two scenarios.
- `diagnose --format json` produces machine-readable output for programmatic assertions.
- Prefer `diagnose` over `simulate --chart` when debugging model behavior — the text output contains all the information needed to reason about trajectory shape without reading a PNG.
- When a user reports "the chart looks wrong", run `diagnose` first to identify which variable has unexpected peaks, phases, or anomalies, then investigate the relevant sector code.
```

## Design Principles

- Analysis functions take `&[f64]` slices, not `SimulationOutput` — testable without simulation
- Text and JSON formatters share the same `SimDiagnostics` struct — single source of truth
- No new dependencies
- Same 6 variables as existing chart/validate — consistent vocabulary
