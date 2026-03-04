use serde::Serialize;
use super::analysis::{PhaseKind, SimDiagnostics};

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

/// Compare two simulation diagnostics (baseline vs comparison) and compute deltas.
pub fn compare(baseline: SimDiagnostics, comparison: SimDiagnostics) -> ComparativeDiagnostics {
    let mut deltas = Vec::new();

    for base_var in &baseline.variables {
        if let Some(comp_var) = comparison.variables.iter().find(|v| v.name == base_var.name) {
            let peak_value_change = comp_var.peak.value - base_var.peak.value;
            let peak_value_pct_change = if base_var.peak.value.abs() > 1e-12 {
                (peak_value_change / base_var.peak.value) * 100.0
            } else {
                0.0
            };
            let peak_year_shift = comp_var.peak.year - base_var.peak.year;
            let final_value_change = comp_var.final_value - base_var.final_value;

            // Compare phase kind sequences
            let base_kinds: Vec<&PhaseKind> = base_var.phases.iter().map(|p| &p.kind).collect();
            let comp_kinds: Vec<&PhaseKind> = comp_var.phases.iter().map(|p| &p.kind).collect();
            let trajectory_changed = base_kinds != comp_kinds;

            let phase_diff = if trajectory_changed {
                let base_str = phase_kinds_to_string(&base_kinds);
                let comp_str = phase_kinds_to_string(&comp_kinds);
                format!("{} -> {}", base_str, comp_str)
            } else {
                String::new()
            };

            deltas.push(VariableDelta {
                name: base_var.name.clone(),
                peak_value_change,
                peak_value_pct_change,
                peak_year_shift,
                final_value_change,
                trajectory_changed,
                phase_diff,
            });
        }
    }

    ComparativeDiagnostics {
        baseline,
        comparison,
        deltas,
    }
}

fn phase_kinds_to_string(kinds: &[&PhaseKind]) -> String {
    kinds
        .iter()
        .map(|k| match k {
            PhaseKind::Growing => "Growing",
            PhaseKind::Declining => "Declining",
            PhaseKind::Plateau => "Plateau",
        })
        .collect::<Vec<_>>()
        .join("/")
}

// REQ: REQ-003
#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnose::analysis::*;

    fn make_diag(preset: &str, peak_val: f64, peak_yr: f64, final_val: f64, phases: Vec<Phase>) -> SimDiagnostics {
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
        let baseline = make_diag(
            "bau",
            7.2e9,
            2032.0,
            3.5e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2032.0, start_value: 1.6e9, end_value: 7.2e9, avg_annual_rate: 0.012 },
                Phase { kind: PhaseKind::Declining, start_year: 2032.0, end_year: 2100.0, start_value: 7.2e9, end_value: 3.5e9, avg_annual_rate: -0.011 },
            ],
        );
        let comparison = make_diag(
            "technology",
            9.0e9,
            2044.0,
            6.0e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2044.0, start_value: 1.6e9, end_value: 9.0e9, avg_annual_rate: 0.013 },
                Phase { kind: PhaseKind::Declining, start_year: 2044.0, end_year: 2100.0, start_value: 9.0e9, end_value: 6.0e9, avg_annual_rate: -0.007 },
            ],
        );

        let result = compare(baseline, comparison);
        assert_eq!(result.deltas.len(), 1);
        let d = &result.deltas[0];
        assert_eq!(d.name, "Population");
        assert!(d.peak_year_shift > 0.0, "peak should be later in tech scenario");
        assert_eq!(d.peak_year_shift, 12.0);
        assert!(d.peak_value_change > 0.0, "peak should be higher in tech scenario");
        assert!(d.final_value_change > 0.0, "final value should be higher in tech scenario");
    }

    #[test]
    fn compare_no_nan_in_deltas() {
        let baseline = make_diag(
            "bau",
            7.2e9,
            2032.0,
            3.5e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2032.0, start_value: 1.6e9, end_value: 7.2e9, avg_annual_rate: 0.012 },
            ],
        );
        let comparison = make_diag(
            "technology",
            9.0e9,
            2044.0,
            6.0e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2044.0, start_value: 1.6e9, end_value: 9.0e9, avg_annual_rate: 0.013 },
            ],
        );

        let result = compare(baseline, comparison);
        for d in &result.deltas {
            assert!(!d.peak_value_change.is_nan(), "peak_value_change is NaN");
            assert!(!d.peak_value_pct_change.is_nan(), "peak_value_pct_change is NaN");
            assert!(!d.peak_year_shift.is_nan(), "peak_year_shift is NaN");
            assert!(!d.final_value_change.is_nan(), "final_value_change is NaN");
        }
    }

    #[test]
    fn compare_detects_trajectory_change() {
        let baseline = make_diag(
            "bau",
            7.2e9,
            2032.0,
            3.5e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2032.0, start_value: 1.6e9, end_value: 7.2e9, avg_annual_rate: 0.012 },
                Phase { kind: PhaseKind::Declining, start_year: 2032.0, end_year: 2100.0, start_value: 7.2e9, end_value: 3.5e9, avg_annual_rate: -0.011 },
            ],
        );
        let comparison = make_diag(
            "stabilized",
            8.0e9,
            2060.0,
            7.5e9,
            vec![
                Phase { kind: PhaseKind::Growing, start_year: 1900.0, end_year: 2060.0, start_value: 1.6e9, end_value: 8.0e9, avg_annual_rate: 0.010 },
                Phase { kind: PhaseKind::Plateau, start_year: 2060.0, end_year: 2100.0, start_value: 8.0e9, end_value: 7.5e9, avg_annual_rate: -0.001 },
            ],
        );

        let result = compare(baseline, comparison);
        let d = &result.deltas[0];
        assert!(d.trajectory_changed, "trajectory should have changed");
        assert!(!d.phase_diff.is_empty(), "phase_diff should be non-empty");
        assert!(d.phase_diff.contains("Growing"), "phase_diff should mention Growing");
    }
}
