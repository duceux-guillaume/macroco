use super::analysis::{SimDiagnostics, StabilityReport};
use super::compare::ComparativeDiagnostics;

/// Format a SimDiagnostics as pretty-printed JSON.
pub fn format_json(diag: &SimDiagnostics) -> String {
    serde_json::to_string_pretty(diag).expect("failed to serialize SimDiagnostics")
}

/// Format a ComparativeDiagnostics as pretty-printed JSON.
pub fn format_json_comparative(comp: &ComparativeDiagnostics) -> String {
    serde_json::to_string_pretty(comp).expect("failed to serialize ComparativeDiagnostics")
}

/// Format a StabilityReport as pretty-printed JSON.
pub fn format_json_stability(report: &StabilityReport) -> String {
    serde_json::to_string_pretty(report).expect("failed to serialize StabilityReport")
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
                phases: vec![Phase {
                    kind: PhaseKind::Growing,
                    start_year: 1900.0,
                    end_year: 2032.0,
                    start_value: 1.6e9,
                    end_value: 7.2e9,
                    avg_annual_rate: 0.012,
                }],
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
    fn json_parses_and_has_preset_name() {
        let diag = make_test_diagnostics();
        let json_str = format_json(&diag);
        let val: serde_json::Value =
            serde_json::from_str(&json_str).expect("invalid JSON output");
        assert_eq!(val["preset_name"], "bau");
    }

    #[test]
    fn json_has_num_steps() {
        let diag = make_test_diagnostics();
        let json_str = format_json(&diag);
        let val: serde_json::Value =
            serde_json::from_str(&json_str).expect("invalid JSON output");
        assert_eq!(val["num_steps"], 201);
    }

    #[test]
    fn json_has_variables_array() {
        let diag = make_test_diagnostics();
        let json_str = format_json(&diag);
        let val: serde_json::Value =
            serde_json::from_str(&json_str).expect("invalid JSON output");
        let variables = val["variables"].as_array().expect("variables should be array");
        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0]["name"], "Population");
    }
}
