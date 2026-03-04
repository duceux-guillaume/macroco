use super::analysis::{PhaseKind, SimDiagnostics, VariableDiagnostics};
use super::compare::ComparativeDiagnostics;
use std::fmt::Write;

/// Format a SimDiagnostics as human-readable text.
pub fn format_text(diag: &SimDiagnostics) -> String {
    let mut out = String::new();

    writeln!(
        out,
        "=== Simulation Diagnostics: {} ===",
        diag.preset_name
    )
    .unwrap();
    writeln!(
        out,
        "Time: {:.0}-{:.0}, dt={:.1}yr, {} steps\n",
        diag.time_range.0, diag.time_range.1, diag.dt, diag.num_steps
    )
    .unwrap();

    for var in &diag.variables {
        format_variable(&mut out, var, diag.time_range);
    }

    writeln!(out, "-- Anomalies ----------------------------------------").unwrap();
    if diag.anomalies.is_empty() {
        writeln!(out, "  None detected.").unwrap();
    } else {
        for a in &diag.anomalies {
            writeln!(
                out,
                "  {:?} in {} at year {:.0}: {:.3e}",
                a.kind, a.variable, a.year, a.value
            )
            .unwrap();
        }
    }

    out
}

fn format_variable(out: &mut String, var: &VariableDiagnostics, time_range: (f64, f64)) {
    writeln!(
        out,
        "-- {} ------------------------------------------------",
        var.name
    )
    .unwrap();
    writeln!(out, "  Initial ({:.0}):  {:.3e}", time_range.0, var.initial).unwrap();
    writeln!(
        out,
        "  Peak:            {:.3e}  at year {:.0}",
        var.peak.value, var.peak.year
    )
    .unwrap();
    writeln!(
        out,
        "  Trough:          {:.3e}  at year {:.0}",
        var.trough.value, var.trough.year
    )
    .unwrap();
    writeln!(
        out,
        "  Final ({:.0}):    {:.3e}",
        time_range.1, var.final_value
    )
    .unwrap();

    // Phases
    if !var.phases.is_empty() {
        let phase_strs: Vec<String> = var
            .phases
            .iter()
            .map(|p| {
                let kind_str = match p.kind {
                    PhaseKind::Growing => "Growing",
                    PhaseKind::Declining => "Declining",
                    PhaseKind::Plateau => "Plateau",
                };
                format!(
                    "{} {:.0}-{:.0} ({:+.1}%/yr avg)",
                    kind_str,
                    p.start_year,
                    p.end_year,
                    p.avg_annual_rate * 100.0
                )
            })
            .collect();
        writeln!(out, "  Phases:          {}", phase_strs.join(" -> ")).unwrap();
    }

    // Growth and decline rates
    if var.max_growth_rate.value > 0.0 {
        writeln!(
            out,
            "  Max growth rate: {:+.1}%/yr at {:.0}",
            var.max_growth_rate.value * 100.0,
            var.max_growth_rate.year
        )
        .unwrap();
    }
    if var.max_decline_rate.value < 0.0 {
        writeln!(
            out,
            "  Max decline rate: {:.1}%/yr at {:.0}",
            var.max_decline_rate.value * 100.0,
            var.max_decline_rate.year
        )
        .unwrap();
    }

    writeln!(out).unwrap();
}

/// Format a ComparativeDiagnostics as human-readable text.
pub fn format_text_comparative(comp: &ComparativeDiagnostics) -> String {
    let mut out = String::new();

    writeln!(
        out,
        "=== Comparative Diagnostics: {} vs {} ===\n",
        comp.baseline.preset_name, comp.comparison.preset_name
    )
    .unwrap();

    for delta in &comp.deltas {
        // Find matching variables in baseline and comparison
        let base_var = comp.baseline.variables.iter().find(|v| v.name == delta.name);
        let comp_var = comp
            .comparison
            .variables
            .iter()
            .find(|v| v.name == delta.name);

        writeln!(
            out,
            "-- {} ------------------------------------------------",
            delta.name
        )
        .unwrap();

        if let Some(bv) = base_var {
            writeln!(
                out,
                "  Baseline ({}):     peak {:.3e} at {:.0}, final {:.3e}",
                comp.baseline.preset_name, bv.peak.value, bv.peak.year, bv.final_value
            )
            .unwrap();
        }
        if let Some(cv) = comp_var {
            writeln!(
                out,
                "  Comparison ({}):  peak {:.3e} at {:.0}, final {:.3e}",
                comp.comparison.preset_name, cv.peak.value, cv.peak.year, cv.final_value
            )
            .unwrap();
        }

        writeln!(
            out,
            "  D peak:  {:+.3e} ({:+.1}%), {:.0} years {}",
            delta.peak_value_change,
            delta.peak_value_pct_change,
            delta.peak_year_shift.abs(),
            if delta.peak_year_shift > 0.0 {
                "later"
            } else if delta.peak_year_shift < 0.0 {
                "earlier"
            } else {
                "same"
            }
        )
        .unwrap();
        writeln!(out, "  D final: {:+.3e}", delta.final_value_change).unwrap();

        if delta.trajectory_changed {
            writeln!(out, "  Trajectory: CHANGED — {}", delta.phase_diff).unwrap();
        } else {
            writeln!(out, "  Trajectory: same pattern").unwrap();
        }

        writeln!(out).unwrap();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnose::analysis::*;

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
                peak: ValueAtYear {
                    value: 7.2e9,
                    year: 2032.0,
                },
                trough: ValueAtYear {
                    value: 3.5e9,
                    year: 2100.0,
                },
                phases: vec![
                    Phase {
                        kind: PhaseKind::Growing,
                        start_year: 1900.0,
                        end_year: 2032.0,
                        start_value: 1.6e9,
                        end_value: 7.2e9,
                        avg_annual_rate: 0.012,
                    },
                    Phase {
                        kind: PhaseKind::Declining,
                        start_year: 2032.0,
                        end_year: 2100.0,
                        start_value: 7.2e9,
                        end_value: 3.5e9,
                        avg_annual_rate: -0.011,
                    },
                ],
                inflection_points: vec![],
                is_monotonic: false,
                max_growth_rate: ValueAtYear {
                    value: 0.021,
                    year: 1968.0,
                },
                max_decline_rate: ValueAtYear {
                    value: -0.023,
                    year: 2058.0,
                },
            }],
            anomalies: vec![],
        }
    }

    #[test]
    fn text_contains_header() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(
            text.contains("Simulation Diagnostics: bau"),
            "missing header"
        );
        assert!(text.contains("1900"), "missing start year");
        assert!(text.contains("2100"), "missing end year");
    }

    #[test]
    fn text_contains_variable_section() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Population"), "missing variable name");
        assert!(text.contains("Peak:"), "missing Peak label");
        assert!(text.contains("2032"), "missing peak year");
        assert!(text.contains("Phases:"), "missing Phases label");
        assert!(text.contains("Growing"), "missing Growing phase");
        assert!(text.contains("Declining"), "missing Declining phase");
    }

    #[test]
    fn text_contains_anomaly_section() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Anomalies"), "missing Anomalies header");
        assert!(text.contains("None detected"), "missing none detected");
    }

    #[test]
    fn text_contains_growth_and_decline_rates() {
        let diag = make_test_diagnostics();
        let text = format_text(&diag);
        assert!(text.contains("Max growth rate:"), "missing growth rate");
        assert!(text.contains("Max decline rate:"), "missing decline rate");
        assert!(text.contains("1968"), "missing growth year");
        assert!(text.contains("2058"), "missing decline year");
    }
}
