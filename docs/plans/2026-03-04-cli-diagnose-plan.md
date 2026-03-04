# CLI Diagnose Command — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `world3-cli diagnose` subcommand that outputs structured text/JSON diagnostics for simulation debugging without visual chart inspection.

**Architecture:** New `diagnose/` module in `world3-cli` crate with pure analysis functions (`&[f64]` → metrics), two formatters (text, JSON), and comparison logic. Wired into `main.rs` as a new `Commands::Diagnose` variant.

**Tech Stack:** Rust, clap, serde/serde_json (all already available). No new dependencies.

---

### Task 1: Scaffold the diagnose module with data types

**Files:**
- Create: `crates/world3-cli/src/diagnose/mod.rs`
- Create: `crates/world3-cli/src/diagnose/analysis.rs`
- Create: `crates/world3-cli/src/diagnose/format_text.rs`
- Create: `crates/world3-cli/src/diagnose/format_json.rs`
- Create: `crates/world3-cli/src/diagnose/compare.rs`
- Modify: `crates/world3-cli/src/main.rs:1` (add `mod diagnose;`)

**Step 1: Create the data model structs**

Create `crates/world3-cli/src/diagnose/analysis.rs` with all data types:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ValueAtYear {
    pub value: f64,
    pub year: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PhaseKind {
    Growing,
    Declining,
    Plateau,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phase {
    pub kind: PhaseKind,
    pub start_year: f64,
    pub end_year: f64,
    pub start_value: f64,
    pub end_value: f64,
    pub avg_annual_rate: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AnomalyKind {
    Negative,
    NaN,
    Inf,
    Discontinuity,
}

#[derive(Debug, Clone, Serialize)]
pub struct Anomaly {
    pub year: f64,
    pub variable: String,
    pub kind: AnomalyKind,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableDiagnostics {
    pub name: String,
    pub unit: String,
    pub initial: f64,
    pub final_value: f64,
    pub peak: ValueAtYear,
    pub trough: ValueAtYear,
    pub phases: Vec<Phase>,
    pub inflection_points: Vec<ValueAtYear>,
    pub is_monotonic: bool,
    pub max_growth_rate: ValueAtYear,
    pub max_decline_rate: ValueAtYear,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimDiagnostics {
    pub preset_name: String,
    pub time_range: (f64, f64),
    pub dt: f64,
    pub num_steps: usize,
    pub variables: Vec<VariableDiagnostics>,
    pub anomalies: Vec<Anomaly>,
}
```

**Step 2: Create stub module files**

Create `crates/world3-cli/src/diagnose/mod.rs`:

```rust
pub mod analysis;
pub mod compare;
pub mod format_json;
pub mod format_text;
```

Create `crates/world3-cli/src/diagnose/compare.rs`:

```rust
use serde::Serialize;
use super::analysis::SimDiagnostics;

#[derive(Debug, Clone, Serialize)]
pub struct VariableDelta {
    pub name: String,
    pub peak_value_change: f64,
    pub peak_value_pct_change: f64,
    pub peak_year_shift: f64,
    pub final_value_change: f64,
    pub trajectory_changed: bool,
    pub phase_diff: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparativeDiagnostics {
    pub baseline: SimDiagnostics,
    pub comparison: SimDiagnostics,
    pub deltas: Vec<VariableDelta>,
}
```

Create empty `crates/world3-cli/src/diagnose/format_text.rs` and `format_json.rs`:

```rust
// format_text.rs — will be implemented in Task 7
// format_json.rs — will be implemented in Task 8
```

Add `mod diagnose;` to `crates/world3-cli/src/main.rs` after the existing `use` imports.

**Step 3: Verify it compiles**

Run: `cargo build --bin world3-cli`
Expected: compiles with no errors (types are defined but unused — that's ok)

**Step 4: Commit**

```bash
git add crates/world3-cli/src/diagnose/
git commit -m "feat(cli): scaffold diagnose module with data types"
```

---

### Task 2: Peak/trough detection with TDD

**Files:**
- Modify: `crates/world3-cli/src/diagnose/analysis.rs`

**Step 1: Write failing tests for peak and trough detection**

Add to `analysis.rs`:

```rust
/// Find the maximum value and corresponding year in the series.
pub fn find_peak(years: &[f64], values: &[f64]) -> ValueAtYear {
    todo!()
}

/// Find the minimum value after the peak year.
/// If no trough exists (peak is at the end), returns the final value.
pub fn find_trough_after_peak(years: &[f64], values: &[f64], peak_year: f64) -> ValueAtYear {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_of_grow_then_decline() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        // Peaks at year 1950 (index 50)
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();
        let peak = find_peak(&years, &values);
        assert_eq!(peak.year, 1950.0);
        assert_eq!(peak.value, 2500.0);
    }

    #[test]
    fn peak_of_monotonically_increasing() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| y - 1900.0).collect();
        let peak = find_peak(&years, &values);
        assert_eq!(peak.year, 2000.0);
        assert_eq!(peak.value, 100.0);
    }

    #[test]
    fn trough_after_peak() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        // Peaks at 1950, trough at 2000
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();
        let trough = find_trough_after_peak(&years, &values, 1950.0);
        assert_eq!(trough.year, 2000.0);
        // Value at 2000 = -(2000-1950)^2 + 2500 = 0
        assert_eq!(trough.value, 0.0);
    }

    #[test]
    fn trough_when_peak_at_end() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| y - 1900.0).collect();
        let peak = find_peak(&years, &values);
        let trough = find_trough_after_peak(&years, &values, peak.year);
        // Trough is also at the end (no decline)
        assert_eq!(trough.year, 2000.0);
        assert_eq!(trough.value, 100.0);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: FAIL — `todo!()` panics

**Step 3: Implement peak and trough detection**

```rust
pub fn find_peak(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert_eq!(years.len(), values.len());
    assert!(!years.is_empty());
    let (idx, &max_val) = values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    ValueAtYear { value: max_val, year: years[idx] }
}

pub fn find_trough_after_peak(years: &[f64], values: &[f64], peak_year: f64) -> ValueAtYear {
    assert_eq!(years.len(), values.len());
    let start_idx = years.iter().position(|&y| y >= peak_year).unwrap_or(0);
    let (idx, &min_val) = values[start_idx..]
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    ValueAtYear { value: min_val, year: years[start_idx + idx] }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: all 4 tests PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/analysis.rs
git commit -m "feat(cli): add peak/trough detection with tests"
```

---

### Task 3: Phase segmentation with TDD

**Files:**
- Modify: `crates/world3-cli/src/diagnose/analysis.rs`

**Step 1: Write failing tests for phase segmentation**

Add tests and a `todo!()` stub:

```rust
/// Minimum absolute annual rate to classify as Growing or Declining (vs Plateau).
const PLATEAU_THRESHOLD: f64 = 0.001; // 0.1%/yr

/// Segment a time series into Growing, Declining, and Plateau phases.
/// Consecutive years with the same direction are merged into one phase.
pub fn segment_phases(years: &[f64], values: &[f64]) -> Vec<Phase> {
    todo!()
}

/// Returns true if the series is monotonically non-increasing or non-decreasing.
pub fn is_monotonic(values: &[f64]) -> bool {
    todo!()
}
```

Tests:

```rust
    #[test]
    fn phases_grow_then_decline() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();
        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].kind, PhaseKind::Growing);
        assert_eq!(phases[0].start_year, 1900.0);
        assert_eq!(phases[0].end_year, 1950.0);
        assert_eq!(phases[1].kind, PhaseKind::Declining);
        assert_eq!(phases[1].start_year, 1950.0);
        assert_eq!(phases[1].end_year, 2000.0);
    }

    #[test]
    fn phases_monotonic_decline() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| 1.0 - (y - 1900.0) / 200.0).collect();
        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].kind, PhaseKind::Declining);
    }

    #[test]
    fn monotonic_true_for_declining() {
        let values: Vec<f64> = (0..100).map(|i| 1.0 - i as f64 * 0.01).collect();
        assert!(is_monotonic(&values));
    }

    #[test]
    fn monotonic_false_for_up_then_down() {
        let values: Vec<f64> = (0..100).map(|i| {
            -(i as f64 - 50.0).powi(2) + 2500.0
        }).collect();
        assert!(!is_monotonic(&values));
    }

    #[test]
    fn phases_have_avg_annual_rate() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        // Linear growth from 100 to 200 over 100 years
        let values: Vec<f64> = years.iter().map(|&y| 100.0 + (y - 1900.0)).collect();
        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].kind, PhaseKind::Growing);
        // avg rate = (200-100) / 100 / 150 = ~0.67%/yr (using midpoint)
        // Simpler: (end - start) / duration / start = 100/100/100 = 1.0%/yr
        assert!(phases[0].avg_annual_rate > 0.0);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: FAIL on new tests, previous tests still pass

**Step 3: Implement phase segmentation**

```rust
pub fn segment_phases(years: &[f64], values: &[f64]) -> Vec<Phase> {
    assert_eq!(years.len(), values.len());
    if years.len() < 2 {
        return vec![];
    }

    let mut phases: Vec<Phase> = Vec::new();
    let mut phase_start_idx = 0;

    // Classify each step as growing, declining, or plateau
    let classify = |v0: f64, v1: f64, dt: f64| -> PhaseKind {
        if v0 == 0.0 && v1 == 0.0 {
            return PhaseKind::Plateau;
        }
        let base = if v0.abs() > 1e-30 { v0 } else { v1 };
        let rate = (v1 - v0) / dt / base.abs();
        if rate > PLATEAU_THRESHOLD {
            PhaseKind::Growing
        } else if rate < -PLATEAU_THRESHOLD {
            PhaseKind::Declining
        } else {
            PhaseKind::Plateau
        }
    };

    let mut current_kind = classify(values[0], values[1], years[1] - years[0]);

    for i in 1..years.len() - 1 {
        let kind = classify(values[i], values[i + 1], years[i + 1] - years[i]);
        if kind != current_kind {
            // Close the current phase at this boundary
            let duration = years[i] - years[phase_start_idx];
            let start_val = values[phase_start_idx];
            let end_val = values[i];
            let avg_rate = if duration > 0.0 && start_val.abs() > 1e-30 {
                (end_val - start_val) / duration / start_val.abs()
            } else {
                0.0
            };
            phases.push(Phase {
                kind: current_kind.clone(),
                start_year: years[phase_start_idx],
                end_year: years[i],
                start_value: start_val,
                end_value: end_val,
                avg_annual_rate: avg_rate,
            });
            phase_start_idx = i;
            current_kind = kind;
        }
    }

    // Close final phase
    let last = years.len() - 1;
    let duration = years[last] - years[phase_start_idx];
    let start_val = values[phase_start_idx];
    let end_val = values[last];
    let avg_rate = if duration > 0.0 && start_val.abs() > 1e-30 {
        (end_val - start_val) / duration / start_val.abs()
    } else {
        0.0
    };
    phases.push(Phase {
        kind: current_kind,
        start_year: years[phase_start_idx],
        end_year: years[last],
        start_value: start_val,
        end_value: end_val,
        avg_annual_rate: avg_rate,
    });

    phases
}

pub fn is_monotonic(values: &[f64]) -> bool {
    if values.len() < 2 {
        return true;
    }
    let increasing = values.windows(2).all(|w| w[1] >= w[0] - 1e-10);
    let decreasing = values.windows(2).all(|w| w[1] <= w[0] + 1e-10);
    increasing || decreasing
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: all tests PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/analysis.rs
git commit -m "feat(cli): add phase segmentation and monotonicity detection"
```

---

### Task 4: Anomaly detection and rate computation with TDD

**Files:**
- Modify: `crates/world3-cli/src/diagnose/analysis.rs`

**Step 1: Write failing tests**

Stubs:

```rust
/// Detect anomalies (NaN, Inf, negative, discontinuity) in a series.
pub fn detect_anomalies(name: &str, years: &[f64], values: &[f64]) -> Vec<Anomaly> {
    todo!()
}

/// Find the year with the maximum year-over-year growth rate (positive).
pub fn max_growth_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    todo!()
}

/// Find the year with the maximum year-over-year decline rate (negative).
pub fn max_decline_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    todo!()
}

/// Find inflection points where the second derivative changes sign.
pub fn find_inflection_points(years: &[f64], values: &[f64]) -> Vec<ValueAtYear> {
    todo!()
}
```

Tests:

```rust
    #[test]
    fn anomaly_detects_nan() {
        let years = vec![1900.0, 1901.0, 1902.0];
        let values = vec![1.0, f64::NAN, 2.0];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::NaN);
        assert_eq!(anomalies[0].year, 1901.0);
    }

    #[test]
    fn anomaly_detects_negative() {
        let years = vec![1900.0, 1901.0, 1902.0];
        let values = vec![1.0, -0.5, 2.0];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::Negative);
    }

    #[test]
    fn anomaly_detects_inf() {
        let years = vec![1900.0, 1901.0];
        let values = vec![1.0, f64::INFINITY];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::Inf);
    }

    #[test]
    fn no_anomalies_in_clean_series() {
        let years: Vec<f64> = (0..100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| y - 1900.0).collect();
        let anomalies = detect_anomalies("test", &years, &values);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn max_growth_rate_of_linear() {
        let years: Vec<f64> = (0..=10).map(|i| 1900.0 + i as f64).collect();
        // Exponential-ish: doubles at year 5
        let values: Vec<f64> = years.iter().map(|&y| (0.1 * (y - 1900.0)).exp()).collect();
        let rate = max_growth_rate(&years, &values);
        assert!(rate.value > 0.0);
    }

    #[test]
    fn max_decline_rate_of_declining() {
        let years: Vec<f64> = (0..=10).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| (-0.1 * (y - 1900.0)).exp()).collect();
        let rate = max_decline_rate(&years, &values);
        assert!(rate.value < 0.0);
    }

    #[test]
    fn inflection_point_of_sigmoid() {
        let years: Vec<f64> = (0..=200).map(|i| 1900.0 + i as f64).collect();
        // Sigmoid centered at 2000
        let values: Vec<f64> = years.iter().map(|&y| {
            1.0 / (1.0 + (-0.05 * (y - 2000.0)).exp())
        }).collect();
        let inflections = find_inflection_points(&years, &values);
        assert!(!inflections.is_empty());
        // Should be near year 2000
        assert!((inflections[0].year - 2000.0).abs() < 5.0);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: FAIL on new tests

**Step 3: Implement anomaly detection and rate functions**

```rust
pub fn detect_anomalies(name: &str, years: &[f64], values: &[f64]) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    for (i, &v) in values.iter().enumerate() {
        if v.is_nan() {
            anomalies.push(Anomaly {
                year: years[i], variable: name.to_string(),
                kind: AnomalyKind::NaN, value: v,
            });
        } else if v.is_infinite() {
            anomalies.push(Anomaly {
                year: years[i], variable: name.to_string(),
                kind: AnomalyKind::Inf, value: v,
            });
        } else if v < 0.0 {
            anomalies.push(Anomaly {
                year: years[i], variable: name.to_string(),
                kind: AnomalyKind::Negative, value: v,
            });
        }
    }
    anomalies
}

pub fn max_growth_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert!(years.len() >= 2);
    let mut best = ValueAtYear { value: f64::NEG_INFINITY, year: years[0] };
    for i in 0..years.len() - 1 {
        let dt = years[i + 1] - years[i];
        if dt <= 0.0 || values[i].abs() < 1e-30 { continue; }
        let rate = (values[i + 1] - values[i]) / dt / values[i].abs();
        if rate > best.value {
            best = ValueAtYear { value: rate, year: years[i + 1] };
        }
    }
    best
}

pub fn max_decline_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert!(years.len() >= 2);
    let mut best = ValueAtYear { value: f64::INFINITY, year: years[0] };
    for i in 0..years.len() - 1 {
        let dt = years[i + 1] - years[i];
        if dt <= 0.0 || values[i].abs() < 1e-30 { continue; }
        let rate = (values[i + 1] - values[i]) / dt / values[i].abs();
        if rate < best.value {
            best = ValueAtYear { value: rate, year: years[i + 1] };
        }
    }
    best
}

pub fn find_inflection_points(years: &[f64], values: &[f64]) -> Vec<ValueAtYear> {
    if values.len() < 3 { return vec![]; }
    let mut inflections = Vec::new();
    // Compute second differences and look for sign changes
    let mut prev_d2 = (values[2] - 2.0 * values[1] + values[0]);
    for i in 2..values.len() - 1 {
        let d2 = values[i + 1] - 2.0 * values[i] + values[i - 1];
        if prev_d2.signum() != d2.signum() && prev_d2.signum() != 0.0 && d2.signum() != 0.0 {
            inflections.push(ValueAtYear { value: values[i], year: years[i] });
        }
        prev_d2 = d2;
    }
    inflections
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --bin world3-cli -- diagnose::analysis`
Expected: all tests PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/analysis.rs
git commit -m "feat(cli): add anomaly detection, rate computation, inflection points"
```

---

### Task 5: Build the full `analyze_variable` and `run_analysis` functions

**Files:**
- Modify: `crates/world3-cli/src/diagnose/analysis.rs`
- Modify: `crates/world3-cli/src/diagnose/mod.rs`

**Step 1: Write failing test for `analyze_variable`**

Add to `analysis.rs`:

```rust
/// Analyze a single time series variable and produce its diagnostics.
pub fn analyze_variable(
    name: &str,
    unit: &str,
    years: &[f64],
    values: &[f64],
) -> VariableDiagnostics {
    todo!()
}
```

Test:

```rust
    #[test]
    fn analyze_variable_grow_then_decline() {
        let years: Vec<f64> = (0..=100).map(|i| 1900.0 + i as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();
        let diag = analyze_variable("Test", "units", &years, &values);
        assert_eq!(diag.name, "Test");
        assert_eq!(diag.peak.year, 1950.0);
        assert_eq!(diag.peak.value, 2500.0);
        assert!(!diag.is_monotonic);
        assert!(diag.phases.len() >= 2);
        assert_eq!(diag.initial, values[0]);
        assert_eq!(diag.final_value, *values.last().unwrap());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --bin world3-cli -- diagnose::analysis::tests::analyze_variable`
Expected: FAIL

**Step 3: Implement `analyze_variable`**

```rust
pub fn analyze_variable(
    name: &str,
    unit: &str,
    years: &[f64],
    values: &[f64],
) -> VariableDiagnostics {
    let peak = find_peak(years, values);
    let trough = find_trough_after_peak(years, values, peak.year);
    let phases = segment_phases(years, values);
    let inflection_points = find_inflection_points(years, values);
    let monotonic = is_monotonic(values);
    let growth = max_growth_rate(years, values);
    let decline = max_decline_rate(years, values);

    VariableDiagnostics {
        name: name.to_string(),
        unit: unit.to_string(),
        initial: values[0],
        final_value: *values.last().unwrap_or(&0.0),
        peak,
        trough,
        phases,
        inflection_points,
        is_monotonic: monotonic,
        max_growth_rate: growth,
        max_decline_rate: decline,
    }
}
```

**Step 4: Implement `run_analysis` in `mod.rs`**

This is the entry point that runs a simulation and produces `SimDiagnostics`:

```rust
// In mod.rs:
use anyhow::Result;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

pub use analysis::SimDiagnostics;
pub use compare::ComparativeDiagnostics;

/// The 6 variables tracked by the diagnose command.
/// Each tuple: (name, unit, extraction closure).
fn tracked_variables() -> Vec<(&'static str, &'static str, Box<dyn Fn(&WorldState) -> f64>)> {
    vec![
        ("Population", "people", Box::new(|s: &WorldState| s.population.population)),
        ("Food / capita", "kg/person/yr", Box::new(|s: &WorldState| s.agriculture.food_per_capita)),
        ("Industrial output / capita", "USD/person/yr", Box::new(|s: &WorldState| s.capital.industrial_output_per_capita)),
        ("Services / capita", "USD/person/yr", Box::new(|s: &WorldState| s.capital.service_output_per_capita)),
        ("NNR fraction", "fraction", Box::new(|s: &WorldState| s.resources.fraction_remaining)),
        ("Pollution index", "index (1970=1)", Box::new(|s: &WorldState| s.pollution.pollution_index)),
    ]
}

pub fn run_sim(preset_name: &str, start: f64, end: f64, dt: f64) -> Result<SimulationOutput> {
    let mut params = match preset_name {
        "bau" => ScenarioParams::bau(),
        "technology" => ScenarioParams::comprehensive_technology(),
        "stabilized" => ScenarioParams::stabilized_world(),
        other => anyhow::bail!("Unknown preset '{}'", other),
    };
    params.start_year = start;
    params.end_year = end;
    params.time_step = dt;

    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    Ok(SimulationOutput::new(states, params))
}

pub fn run_analysis(preset_name: &str, start: f64, end: f64, dt: f64) -> Result<SimDiagnostics> {
    let sim = run_sim(preset_name, start, end, dt)?;
    Ok(analyze_sim(&sim, preset_name))
}

pub fn analyze_sim(sim: &SimulationOutput, preset_name: &str) -> SimDiagnostics {
    let years: Vec<f64> = sim.states.iter().map(|s| s.time).collect();
    let vars = tracked_variables();

    let mut all_anomalies = Vec::new();
    let mut variables = Vec::new();

    for (name, unit, extractor) in &vars {
        let values: Vec<f64> = sim.states.iter().map(|s| extractor(s)).collect();
        let anomalies = analysis::detect_anomalies(name, &years, &values);
        all_anomalies.extend(anomalies);
        variables.push(analysis::analyze_variable(name, unit, &years, &values));
    }

    SimDiagnostics {
        preset_name: preset_name.to_string(),
        time_range: (
            *years.first().unwrap_or(&0.0),
            *years.last().unwrap_or(&0.0),
        ),
        dt: sim.params.time_step,
        num_steps: sim.states.len(),
        variables,
        anomalies: all_anomalies,
    }
}
```

**Step 5: Run all tests**

Run: `cargo test --bin world3-cli -- diagnose`
Expected: all tests PASS

**Step 6: Commit**

```bash
git add crates/world3-cli/src/diagnose/
git commit -m "feat(cli): add analyze_variable and run_analysis entry points"
```

---

### Task 6: Text formatter

**Files:**
- Modify: `crates/world3-cli/src/diagnose/format_text.rs`

**Step 1: Write failing test**

```rust
use super::analysis::{SimDiagnostics, VariableDiagnostics, ValueAtYear, Phase, PhaseKind};

pub fn format_text(diag: &SimDiagnostics) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_diagnostics() -> SimDiagnostics {
        SimDiagnostics {
            preset_name: "bau".to_string(),
            time_range: (1900.0, 2100.0),
            dt: 1.0,
            num_steps: 201,
            variables: vec![VariableDiagnostics {
                name: "Population".to_string(),
                unit: "people".to_string(),
                initial: 1.6e9,
                final_value: 3.5e9,
                peak: ValueAtYear { value: 7.2e9, year: 2032.0 },
                trough: ValueAtYear { value: 3.5e9, year: 2100.0 },
                phases: vec![
                    Phase {
                        kind: PhaseKind::Growing,
                        start_year: 1900.0, end_year: 2032.0,
                        start_value: 1.6e9, end_value: 7.2e9,
                        avg_annual_rate: 0.012,
                    },
                    Phase {
                        kind: PhaseKind::Declining,
                        start_year: 2032.0, end_year: 2100.0,
                        start_value: 7.2e9, end_value: 3.5e9,
                        avg_annual_rate: -0.011,
                    },
                ],
                inflection_points: vec![],
                is_monotonic: false,
                max_growth_rate: ValueAtYear { value: 0.021, year: 1968.0 },
                max_decline_rate: ValueAtYear { value: -0.023, year: 2058.0 },
            }],
            anomalies: vec![],
        }
    }

    #[test]
    fn text_contains_header() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Simulation Diagnostics: bau"));
        assert!(text.contains("1900"));
        assert!(text.contains("2100"));
    }

    #[test]
    fn text_contains_variable_section() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Population"));
        assert!(text.contains("Peak:"));
        assert!(text.contains("2032"));
        assert!(text.contains("Phases:"));
        assert!(text.contains("Growing"));
        assert!(text.contains("Declining"));
    }

    #[test]
    fn text_contains_anomaly_section() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Anomalies"));
        assert!(text.contains("None detected"));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --bin world3-cli -- diagnose::format_text`
Expected: FAIL

**Step 3: Implement text formatter**

```rust
use super::analysis::{SimDiagnostics, PhaseKind};
use super::compare::ComparativeDiagnostics;

pub fn format_text(diag: &SimDiagnostics) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "=== Simulation Diagnostics: {} ===\n",
        diag.preset_name
    ));
    out.push_str(&format!(
        "Time: {:.0}-{:.0}, dt={:.1}yr, {} steps\n",
        diag.time_range.0, diag.time_range.1, diag.dt, diag.num_steps
    ));

    for var in &diag.variables {
        out.push_str(&format!(
            "\n-- {} {}\n",
            var.name,
            "-".repeat(52usize.saturating_sub(var.name.len()))
        ));
        out.push_str(&format!("  Initial ({:.0}):  {:.3e}\n", diag.time_range.0, var.initial));
        out.push_str(&format!("  Peak:            {:.3e}  at year {:.0}\n", var.peak.value, var.peak.year));
        out.push_str(&format!("  Trough:          {:.3e}  at year {:.0}\n", var.trough.value, var.trough.year));
        out.push_str(&format!("  Final ({:.0}):    {:.3e}\n", diag.time_range.1, var.final_value));

        // Phases
        let phase_strs: Vec<String> = var.phases.iter().map(|p| {
            let kind = match p.kind {
                PhaseKind::Growing => "Growing",
                PhaseKind::Declining => "Declining",
                PhaseKind::Plateau => "Plateau",
            };
            format!("{} {:.0}-{:.0} ({:+.1}%/yr avg)", kind, p.start_year, p.end_year, p.avg_annual_rate * 100.0)
        }).collect();
        out.push_str(&format!("  Phases:          {}\n", phase_strs.join(" -> ")));

        out.push_str(&format!(
            "  Max growth rate: {:+.1}%/yr at {:.0}\n",
            var.max_growth_rate.value * 100.0, var.max_growth_rate.year
        ));
        out.push_str(&format!(
            "  Max decline rate: {:+.1}%/yr at {:.0}\n",
            var.max_decline_rate.value * 100.0, var.max_decline_rate.year
        ));
    }

    out.push_str(&format!(
        "\n-- Anomalies {}\n",
        "-".repeat(40)
    ));
    if diag.anomalies.is_empty() {
        out.push_str("  None detected.\n");
    } else {
        for a in &diag.anomalies {
            out.push_str(&format!(
                "  {:.0}: {} {:?} (value: {:.3e})\n",
                a.year, a.variable, a.kind, a.value
            ));
        }
    }

    out
}

pub fn format_text_comparative(comp: &ComparativeDiagnostics) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "=== Comparative Diagnostics: {} vs {} ===\n",
        comp.baseline.preset_name, comp.comparison.preset_name
    ));
    out.push_str(&format!(
        "Time: {:.0}-{:.0}, dt={:.1}yr\n",
        comp.baseline.time_range.0, comp.baseline.time_range.1, comp.baseline.dt
    ));

    for (i, delta) in comp.deltas.iter().enumerate() {
        let base_var = &comp.baseline.variables[i];
        let comp_var = &comp.comparison.variables[i];
        out.push_str(&format!(
            "\n-- {} {}\n",
            delta.name,
            "-".repeat(52usize.saturating_sub(delta.name.len()))
        ));
        out.push_str(&format!(
            "  Baseline ({}):     peak {:.3e} at {:.0}, final {:.3e}\n",
            comp.baseline.preset_name, base_var.peak.value, base_var.peak.year, base_var.final_value
        ));
        out.push_str(&format!(
            "  Comparison ({}):  peak {:.3e} at {:.0}, final {:.3e}\n",
            comp.comparison.preset_name, comp_var.peak.value, comp_var.peak.year, comp_var.final_value
        ));
        out.push_str(&format!(
            "  D peak:  {:+.3e} ({:+.1}%), {:.0} years {}\n",
            delta.peak_value_change,
            delta.peak_value_pct_change,
            delta.peak_year_shift.abs(),
            if delta.peak_year_shift > 0.0 { "later" } else if delta.peak_year_shift < 0.0 { "earlier" } else { "same" }
        ));
        out.push_str(&format!("  D final: {:+.3e}\n", delta.final_value_change));
        if delta.trajectory_changed {
            out.push_str(&format!("  Trajectory: {}\n", delta.phase_diff));
        }
    }

    out
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --bin world3-cli -- diagnose::format_text`
Expected: all tests PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/format_text.rs
git commit -m "feat(cli): add text formatter for diagnostics"
```

---

### Task 7: JSON formatter

**Files:**
- Modify: `crates/world3-cli/src/diagnose/format_json.rs`

**Step 1: Write failing test**

```rust
use super::analysis::SimDiagnostics;
use super::compare::ComparativeDiagnostics;

pub fn format_json(diag: &SimDiagnostics) -> String {
    todo!()
}

pub fn format_json_comparative(comp: &ComparativeDiagnostics) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analysis::*;

    #[test]
    fn json_is_valid() {
        let diag = SimDiagnostics {
            preset_name: "bau".to_string(),
            time_range: (1900.0, 2100.0),
            dt: 1.0,
            num_steps: 201,
            variables: vec![],
            anomalies: vec![],
        };
        let json = format_json(&diag);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed["preset_name"], "bau");
        assert_eq!(parsed["num_steps"], 201);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --bin world3-cli -- diagnose::format_json`
Expected: FAIL

**Step 3: Implement**

```rust
pub fn format_json(diag: &SimDiagnostics) -> String {
    serde_json::to_string_pretty(diag).expect("failed to serialize diagnostics")
}

pub fn format_json_comparative(comp: &ComparativeDiagnostics) -> String {
    serde_json::to_string_pretty(comp).expect("failed to serialize comparative diagnostics")
}
```

**Step 4: Run tests**

Run: `cargo test --bin world3-cli -- diagnose::format_json`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/format_json.rs
git commit -m "feat(cli): add JSON formatter for diagnostics"
```

---

### Task 8: Comparison logic with TDD

**Files:**
- Modify: `crates/world3-cli/src/diagnose/compare.rs`

**Step 1: Write failing test**

Add to `compare.rs`:

```rust
use super::analysis::SimDiagnostics;

pub fn compare(baseline: SimDiagnostics, comparison: SimDiagnostics) -> ComparativeDiagnostics {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analysis::*;

    fn make_diag(preset: &str, peak_val: f64, peak_yr: f64, final_val: f64, n_phases: usize) -> SimDiagnostics {
        let phases: Vec<Phase> = if n_phases == 1 {
            vec![Phase { kind: PhaseKind::Declining, start_year: 1900.0, end_year: 2100.0, start_value: 1.0, end_value: 0.5, avg_annual_rate: -0.005 }]
        } else {
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: peak_yr, start_value: 1.0, end_value: peak_val, avg_annual_rate: 0.01 },
                Phase { kind: PhaseKind::Declining, start_year: peak_yr, end_year: 2100.0, start_value: peak_val, end_value: final_val, avg_annual_rate: -0.01 },
            ]
        };
        SimDiagnostics {
            preset_name: preset.to_string(),
            time_range: (1900.0, 2100.0),
            dt: 1.0,
            num_steps: 201,
            variables: vec![VariableDiagnostics {
                name: "Population".to_string(),
                unit: "people".to_string(),
                initial: 1.6e9,
                final_value: final_val,
                peak: ValueAtYear { value: peak_val, year: peak_yr },
                trough: ValueAtYear { value: final_val, year: 2100.0 },
                phases,
                inflection_points: vec![],
                is_monotonic: false,
                max_growth_rate: ValueAtYear { value: 0.02, year: 1970.0 },
                max_decline_rate: ValueAtYear { value: -0.02, year: 2060.0 },
            }],
            anomalies: vec![],
        }
    }

    #[test]
    fn compare_detects_peak_shift() {
        let base = make_diag("bau", 7.2e9, 2030.0, 3.5e9, 2);
        let comp = make_diag("tech", 9.0e9, 2045.0, 6.0e9, 2);
        let result = compare(base, comp);
        assert_eq!(result.deltas.len(), 1);
        let d = &result.deltas[0];
        assert!(d.peak_year_shift > 0.0); // peaks later
        assert!(d.peak_value_change > 0.0); // peaks higher
        assert!(d.final_value_change > 0.0); // higher at end
    }

    #[test]
    fn compare_no_nan_in_deltas() {
        let base = make_diag("bau", 7.2e9, 2030.0, 3.5e9, 2);
        let comp = make_diag("tech", 9.0e9, 2045.0, 6.0e9, 2);
        let result = compare(base, comp);
        for d in &result.deltas {
            assert!(!d.peak_value_change.is_nan());
            assert!(!d.peak_value_pct_change.is_nan());
            assert!(!d.peak_year_shift.is_nan());
            assert!(!d.final_value_change.is_nan());
        }
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --bin world3-cli -- diagnose::compare`
Expected: FAIL

**Step 3: Implement comparison**

```rust
pub fn compare(baseline: SimDiagnostics, comparison: SimDiagnostics) -> ComparativeDiagnostics {
    let deltas: Vec<VariableDelta> = baseline.variables.iter().zip(comparison.variables.iter())
        .map(|(b, c)| {
            let peak_value_change = c.peak.value - b.peak.value;
            let peak_value_pct_change = if b.peak.value.abs() > 1e-30 {
                peak_value_change / b.peak.value * 100.0
            } else {
                0.0
            };
            let peak_year_shift = c.peak.year - b.peak.year;
            let final_value_change = c.final_value - b.final_value;

            let b_kinds: Vec<_> = b.phases.iter().map(|p| &p.kind).collect();
            let c_kinds: Vec<_> = c.phases.iter().map(|p| &p.kind).collect();
            let trajectory_changed = b_kinds != c_kinds;

            let phase_diff = if trajectory_changed {
                format!(
                    "Baseline: {} phases ({}) vs Comparison: {} phases ({})",
                    b.phases.len(),
                    b.phases.iter().map(|p| format!("{:?}", p.kind)).collect::<Vec<_>>().join(", "),
                    c.phases.len(),
                    c.phases.iter().map(|p| format!("{:?}", p.kind)).collect::<Vec<_>>().join(", "),
                )
            } else if peak_year_shift.abs() > 1.0 {
                let first_growing = b.phases.iter().find(|p| p.kind == super::analysis::PhaseKind::Growing);
                if let Some(g) = first_growing {
                    let duration = g.end_year - g.start_year;
                    format!("Growing phase {:.0} years vs {:.0} years", duration, duration + peak_year_shift)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            VariableDelta {
                name: b.name.clone(),
                peak_value_change,
                peak_value_pct_change,
                peak_year_shift,
                final_value_change,
                trajectory_changed,
                phase_diff,
            }
        })
        .collect();

    ComparativeDiagnostics { baseline, comparison, deltas }
}
```

**Step 4: Run tests**

Run: `cargo test --bin world3-cli -- diagnose::compare`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/world3-cli/src/diagnose/compare.rs
git commit -m "feat(cli): add comparative diagnostics with delta computation"
```

---

### Task 9: Wire into main.rs CLI

**Files:**
- Modify: `crates/world3-cli/src/main.rs`

**Step 1: Add the `Diagnose` command variant**

Add to the `Commands` enum (after `Presets`):

```rust
    /// Run simulation diagnostics — structured text/JSON analysis for debugging
    Diagnose {
        /// Preset scenario: bau, technology, stabilized
        #[arg(long, default_value = "bau")]
        preset: String,

        /// Compare against another preset
        #[arg(long)]
        compare: Option<String>,

        /// Output format: text or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,

        /// Start year
        #[arg(long, default_value_t = 1900.0)]
        start: f64,

        /// End year
        #[arg(long, default_value_t = 2100.0)]
        end: f64,

        /// Time step (years)
        #[arg(long, default_value_t = 1.0)]
        dt: f64,
    },
```

**Step 2: Add the match arm in `main()`**

After the `Commands::Presets` arm:

```rust
        Commands::Diagnose { preset, compare: compare_preset, format, start, end, dt } => {
            eprintln!("Running diagnostics for '{}'…", preset);
            let diag = diagnose::run_analysis(&preset, start, end, dt)?;

            if let Some(ref comp_name) = compare_preset {
                eprintln!("Running comparison against '{}'…", comp_name);
                let comp_diag = diagnose::run_analysis(comp_name, start, end, dt)?;
                let comparative = diagnose::compare::compare(diag, comp_diag);
                let output = match format.as_str() {
                    "json" => diagnose::format_json::format_json_comparative(&comparative),
                    _ => diagnose::format_text::format_text_comparative(&comparative),
                };
                println!("{}", output);
            } else {
                let output = match format.as_str() {
                    "json" => diagnose::format_json::format_json(&diag),
                    _ => diagnose::format_text::format_text(&diag),
                };
                println!("{}", output);
            }
        }
```

**Step 3: Verify it compiles and runs**

Run: `cargo build --bin world3-cli`
Expected: compiles

Run: `cargo run --bin world3-cli -- diagnose --preset bau 2>/dev/null | head -20`
Expected: text output with header and first variable section

Run: `cargo run --bin world3-cli -- diagnose --preset bau --format json 2>/dev/null | head -5`
Expected: valid JSON starting with `{`

Run: `cargo run --bin world3-cli -- diagnose --preset bau --compare technology 2>/dev/null | head -20`
Expected: comparative text output

**Step 4: Commit**

```bash
git add crates/world3-cli/src/main.rs
git commit -m "feat(cli): wire diagnose subcommand into CLI"
```

---

### Task 10: End-to-end regression tests

**Files:**
- Modify: `crates/world3-cli/src/diagnose/mod.rs`

**Step 1: Write regression tests**

Add to `mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bau_diagnostics_regression() {
        let diag = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU sim failed");

        // Population peaks in expected range
        let pop = &diag.variables[0];
        assert_eq!(pop.name, "Population");
        assert!(pop.peak.year >= 2000.0 && pop.peak.year <= 2070.0,
            "Population peak year {} outside [2000, 2070]", pop.peak.year);
        assert!(pop.peak.value >= 5.0e9 && pop.peak.value <= 12.0e9,
            "Population peak value {:.2e} outside [5B, 12B]", pop.peak.value);
        assert!(!pop.is_monotonic, "Population should not be monotonic");
        assert!(pop.phases.len() >= 2, "Population should have at least 2 phases");

        // NNR is monotonically declining
        let nnr = &diag.variables[4];
        assert_eq!(nnr.name, "NNR fraction");
        assert!(nnr.is_monotonic, "NNR should be monotonically declining");

        // Food per capita should peak
        let food = &diag.variables[1];
        assert_eq!(food.name, "Food / capita");
        assert!(food.peak.value > food.final_value, "Food/cap should peak then decline");

        // IOPC should collapse
        let iopc = &diag.variables[2];
        assert!(iopc.final_value < iopc.peak.value * 0.5,
            "IOPC should collapse by 2100");

        // No anomalies in standard BAU run
        assert!(diag.anomalies.is_empty(),
            "BAU should have no anomalies, found: {:?}", diag.anomalies);
    }

    #[test]
    fn comparative_bau_vs_technology() {
        let base = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU failed");
        let comp = run_analysis("technology", 1900.0, 2100.0, 1.0).expect("Tech failed");
        let result = compare::compare(base, comp);

        assert_eq!(result.deltas.len(), 6, "Should have 6 variable deltas");

        // All deltas computed without NaN
        for d in &result.deltas {
            assert!(!d.peak_value_change.is_nan(), "{} peak_value_change is NaN", d.name);
            assert!(!d.peak_value_pct_change.is_nan(), "{} peak_value_pct_change is NaN", d.name);
            assert!(!d.peak_year_shift.is_nan(), "{} peak_year_shift is NaN", d.name);
            assert!(!d.final_value_change.is_nan(), "{} final_value_change is NaN", d.name);
        }
    }

    #[test]
    fn json_output_roundtrips() {
        let diag = run_analysis("bau", 1900.0, 2100.0, 1.0).expect("BAU failed");
        let json = format_json::format_json(&diag);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert!(parsed["variables"].as_array().unwrap().len() == 6);
        assert!(parsed["anomalies"].as_array().unwrap().is_empty());
    }
}
```

**Step 2: Run tests**

Run: `cargo test --bin world3-cli -- diagnose::tests`
Expected: all 3 tests PASS

**Step 3: Commit**

```bash
git add crates/world3-cli/src/diagnose/mod.rs
git commit -m "test(cli): add end-to-end regression tests for diagnose"
```

---

### Task 11: Agent workflow test script

**Files:**
- Create: `tests/agent-workflow/test_diagnose_workflow.sh`

**Step 1: Write the shell script**

```bash
#!/usr/bin/env bash
set -euo pipefail

# Agent workflow test: proves that the `diagnose` command provides enough
# structured information for a Claude Code agent to debug simulation output
# without visual chart inspection.

CLI="cargo run --release --bin world3-cli --"
PASS=0
FAIL=0

pass() { echo "  PASS  $1"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL  $1"; FAIL=$((FAIL + 1)); }

echo "=== Agent Workflow Test: diagnose command ==="
echo ""

# Build first (suppress output)
echo "Building CLI..."
cargo build --release --bin world3-cli 2>/dev/null

# --- Test 1: Text output contains expected sections ---
echo ""
echo "Test 1: Text output contains expected sections"
output=$($CLI diagnose --preset bau 2>/dev/null)

echo "$output" | grep -q "Simulation Diagnostics" && pass "header present" || fail "header missing"
echo "$output" | grep -q "Population" && pass "Population section" || fail "Population section missing"
echo "$output" | grep -q "Peak:" && pass "Peak line" || fail "Peak line missing"
echo "$output" | grep -q "Phases:" && pass "Phases line" || fail "Phases line missing"
echo "$output" | grep -q "Anomalies" && pass "Anomalies section" || fail "Anomalies section missing"

# --- Test 2: JSON output is valid and queryable ---
echo ""
echo "Test 2: JSON output is valid and queryable"
json=$($CLI diagnose --preset bau --format json 2>/dev/null)

echo "$json" | jq . > /dev/null 2>&1 && pass "valid JSON" || fail "invalid JSON"

var_count=$(echo "$json" | jq '.variables | length')
[ "$var_count" -eq 6 ] && pass "6 variables tracked" || fail "expected 6 variables, got $var_count"

peak_year=$(echo "$json" | jq '.variables[] | select(.name == "Population") | .peak.year')
[ "$(echo "$peak_year >= 2000 && $peak_year <= 2070" | bc)" -eq 1 ] && \
    pass "population peak year $peak_year in [2000, 2070]" || \
    fail "population peak year $peak_year outside range"

anomaly_count=$(echo "$json" | jq '.anomalies | length')
[ "$anomaly_count" -eq 0 ] && pass "no anomalies in BAU" || fail "unexpected anomalies: $anomaly_count"

# --- Test 3: Comparison mode produces deltas ---
echo ""
echo "Test 3: Comparison mode produces deltas"
comp=$($CLI diagnose --preset bau --compare technology 2>/dev/null)

echo "$comp" | grep -q "Comparative Diagnostics" && pass "comp header" || fail "comp header missing"
echo "$comp" | grep -q "D peak:" && pass "delta peak present" || fail "delta peak missing"
echo "$comp" | grep -q "D final:" && pass "delta final present" || fail "delta final missing"

# --- Test 4: Agent can extract specific diagnostics from JSON ---
echo ""
echo "Test 4: Agent debugging scenario"

# Scenario: "Why is population declining?"
decline_phase=$(echo "$json" | jq -r '.variables[] | select(.name == "Population") | .phases[] | select(.kind == "Declining") | "\(.start_year)-\(.end_year)"')
[ -n "$decline_phase" ] && pass "extracted decline phase: $decline_phase" || fail "could not extract decline phase"

# Scenario: "Is NNR monotonically declining?"
nnr_monotonic=$(echo "$json" | jq '.variables[] | select(.name == "NNR fraction") | .is_monotonic')
[ "$nnr_monotonic" = "true" ] && pass "NNR confirmed monotonic" || fail "NNR not monotonic: $nnr_monotonic"

# Scenario: "What's the max pollution level?"
poll_peak=$(echo "$json" | jq '.variables[] | select(.name == "Pollution index") | .peak.value')
[ -n "$poll_peak" ] && pass "pollution peak: $poll_peak" || fail "could not extract pollution peak"

# --- Summary ---
echo ""
echo "=============================="
TOTAL=$((PASS + FAIL))
echo "Results: $PASS/$TOTAL passed"
if [ "$FAIL" -gt 0 ]; then
    echo "FAILED"
    exit 1
else
    echo "ALL PASSED"
fi
```

**Step 2: Make it executable and run it**

Run: `chmod +x tests/agent-workflow/test_diagnose_workflow.sh`
Run: `./tests/agent-workflow/test_diagnose_workflow.sh`
Expected: ALL PASSED

**Step 3: Commit**

```bash
git add tests/agent-workflow/test_diagnose_workflow.sh
git commit -m "test: add agent workflow test for diagnose command"
```

---

### Task 12: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add Debugging Workflow section**

Add the following after the `### Backend Testing` section in CLAUDE.md:

```markdown
### Debugging Workflow
- For simulation debugging, use `cargo run --bin world3-cli -- diagnose` instead of visual chart inspection.
- `diagnose --preset <name>` outputs a structured text report: peaks, troughs, phases, growth rates, anomalies.
- `diagnose --preset <name> --compare <other>` shows side-by-side deltas between two scenarios.
- `diagnose --format json` produces machine-readable output for programmatic assertions.
- Prefer `diagnose` over `simulate --chart` when debugging model behavior — the text output contains all the information needed to reason about trajectory shape without reading a PNG.
- When a user reports "the chart looks wrong", run `diagnose` first to identify which variable has unexpected peaks, phases, or anomalies, then investigate the relevant sector code.
```

Also add to the Commands section:

```bash
# Diagnose simulation output (structured text report)
cargo run --bin world3-cli -- diagnose --preset bau

# Compare two presets
cargo run --bin world3-cli -- diagnose --preset bau --compare technology

# JSON output for programmatic use
cargo run --bin world3-cli -- diagnose --preset bau --format json
```

**Step 2: Verify the docs reference is accurate**

Run: `cargo run --bin world3-cli -- diagnose --help`
Expected: shows help text matching the documented flags

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add debugging workflow with diagnose command to CLAUDE.md"
```

---

### Task 13: Final verification

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests pass (existing + new)

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

**Step 2: Run agent workflow test**

Run: `./tests/agent-workflow/test_diagnose_workflow.sh`
Expected: ALL PASSED

**Step 3: Manual smoke test**

Run: `cargo run --bin world3-cli -- diagnose --preset bau 2>/dev/null`
Verify: readable text output with all 6 variables

Run: `cargo run --bin world3-cli -- diagnose --preset bau --format json 2>/dev/null | jq '.variables[0].name'`
Expected: `"Population"`

Run: `cargo run --bin world3-cli -- diagnose --preset bau --compare technology 2>/dev/null`
Verify: comparative output with delta lines

**Step 4: Final commit if any fixups needed, then done**
