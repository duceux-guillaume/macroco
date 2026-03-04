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
