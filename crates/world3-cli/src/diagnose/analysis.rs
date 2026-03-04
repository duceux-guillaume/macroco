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
