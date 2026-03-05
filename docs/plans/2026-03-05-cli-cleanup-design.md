# CLI Cleanup: Requirements, Validate Re-architecture, Chart Removal

**Date:** 2026-03-05
**Status:** Draft

## Context

The `world3-cli` crate has grown organically. It currently has:
- `simulate` — run a simulation, export CSV or PNG chart
- `validate` — BAU qualitative checks against Meadows 1972
- `diagnose` — structured debugging reports (text/JSON, comparison, stability)
- `presets` — list available presets

Problems:
1. **No clear CLI purpose statement.** REQ-003 says "batch simulation" but the CLI serves four distinct roles (CI/CD, batch processing, debugging, reproducibility) that should be explicitly documented.
2. **`validate` duplicates test logic.** Another PR is already removing it from the CLI. The validation logic should live in `world3-core` as integration tests, with the CLI as an optional thin wrapper.
3. **`diagnose` is undocumented.** No REQ, not in `docs/cli.md`, despite being the primary debugging tool.
4. **Chart rendering is dead weight.** The `--chart` flag and `plotters` dependency duplicate what the frontend does better. REQ-006 should be abandoned.

## Execution Strategy

Work is split into two steps to coordinate with an in-flight PR that removes `validate` from the CLI:

**Step 1 (this branch):** Remove chart feature + rework requirements + update docs.
**Step 2 (after validate PR merges):** Rebase, create `world3-core::validation` module, make CLI `validate` a thin wrapper.

## Design

### Step 1: Chart Removal + Requirements Rework

#### 1a. New CLI Requirements (REQ-033 through REQ-036)

Supersede REQ-003 (overly broad) with four focused requirements that explain *why* the CLI exists:

| REQ | Purpose | CLI Commands | Rationale |
|-----|---------|-------------|-----------|
| REQ-033 | CI/CD validation | `validate` (thin wrapper) | Headless qualitative checks in pipelines |
| REQ-034 | Batch export | `simulate --output` | CSV export for external analysis tools |
| REQ-035 | Simulation debugging | `diagnose` | Structured analysis without browser |
| REQ-036 | Reproducibility | `simulate`, `presets` | Deterministic runs from a single command |

REQ-003 becomes "Superseded by REQ-033..036". REQ-006 (chart) becomes "Abandoned — superseded by frontend D3 charts (REQ-010)".

#### 1b. Chart Removal

Remove:
- `--chart` flag from `Commands::Simulate`
- `render_chart()` function from `main.rs`
- `plotters` import from `main.rs`
- `plotters` from `crates/world3-cli/Cargo.toml`
- `plotters` from workspace `Cargo.toml` (no other crate uses it)
- `docs/chart-output.md`
- `docs/examples/bau_standard_run.png`
- References in `README.md`, `CLAUDE.md`, `docs/quick-start.md`, `docs/cli.md`, `docs/architecture.md`

#### 1c. CLI Documentation Update

Update `docs/cli.md` to:
- Add introductory section explaining the four CLI purposes
- Add `diagnose` subcommand documentation (currently missing)
- Remove `--chart` flag from `simulate`

Update `docs/architecture.md` CLI section to:
- Reference new REQ-033..036 instead of REQ-003, REQ-006
- Remove plotters/chart references
- Add `diagnose` to the component description

#### Step 1 File Impact

| File | Action |
|------|--------|
| `docs/product-requirements.md` | Add REQ-033..036, supersede REQ-003, abandon REQ-006 |
| `docs/architecture.md` | Update CLI section |
| `docs/cli.md` | Rewrite: add intro, add diagnose, remove chart |
| `docs/chart-output.md` | Delete |
| `docs/examples/bau_standard_run.png` | Delete |
| `docs/quick-start.md` | Remove chart image reference |
| `README.md` | Remove chart-output.md link |
| `CLAUDE.md` | Remove chart references, update CLI commands |
| `crates/world3-cli/src/main.rs` | Remove chart flag, render_chart(), plotters import |
| `crates/world3-cli/Cargo.toml` | Remove plotters dependency |
| `Cargo.toml` (workspace) | Remove plotters from workspace deps |
| `docs/traceability-matrix.md` | Regenerate via `python3 scripts/traceability.py` |

### Step 2: Validate Re-architecture (after validate PR merges)

**Prerequisite:** Rebase onto main after the validate-removal PR lands.

#### 2a. Shared Validation Module

Create `world3-core::validation` — a public module with reusable validation logic:

```rust
// world3-core/src/validation.rs
pub struct ValidationCheck {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

pub fn validate_bau(sim: &SimulationOutput) -> Vec<ValidationCheck> { ... }
```

#### 2b. Integration Tests

Create `world3-core/tests/validation.rs` — integration tests that call `validate_bau()` and assert all checks pass. These run via `cargo test -p world3-core`.

#### 2c. CLI Thin Wrapper

Reintroduce `Commands::Validate` in the CLI as a thin wrapper that calls `world3_core::validation::validate_bau()` and prints human-readable PASS/FAIL output. No inline logic.

#### 2d. Documentation

Update `docs/architecture.md` to document the `world3-core::validation` module and the CLI thin-wrapper pattern.

#### Step 2 File Impact

| File | Action |
|------|--------|
| `crates/world3-core/src/lib.rs` | Add `pub mod validation` |
| `crates/world3-core/src/validation.rs` | New: shared validation logic |
| `crates/world3-core/tests/validation.rs` | New: integration tests |
| `crates/world3-cli/src/main.rs` | Add thin `validate` wrapper |
| `docs/architecture.md` | Add validation module docs |
| `docs/traceability-matrix.md` | Regenerate |

## Out of Scope

- Moving `historical_calibration.rs` tests (REQ-026) — these already work fine in `world3-cli`
- Changing `diagnose` implementation — it stays as-is
