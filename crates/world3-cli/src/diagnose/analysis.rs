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

/// Find the peak (maximum) value and its corresponding year.
///
/// Returns the first occurrence if multiple values share the maximum.
/// Panics if the input slices are empty or of different lengths.
pub fn find_peak(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert_eq!(years.len(), values.len(), "years and values must have same length");
    assert!(!years.is_empty(), "input must not be empty");

    let mut best_idx = 0;
    let mut best_val = f64::NEG_INFINITY;
    for (i, &v) in values.iter().enumerate() {
        if v.partial_cmp(&best_val) == Some(std::cmp::Ordering::Greater) {
            best_val = v;
            best_idx = i;
        }
    }
    ValueAtYear { value: best_val, year: years[best_idx] }
}

/// Find the trough (minimum) value occurring after the given peak year.
///
/// If no data exists after `peak_year`, returns the value at the last year.
/// Panics if the input slices are empty or of different lengths.
pub fn find_trough_after_peak(years: &[f64], values: &[f64], peak_year: f64) -> ValueAtYear {
    assert_eq!(years.len(), values.len(), "years and values must have same length");
    assert!(!years.is_empty(), "input must not be empty");

    let mut best_idx = years.len() - 1; // default to last element
    let mut best_val = f64::INFINITY;

    for (i, (&y, &v)) in years.iter().zip(values.iter()).enumerate() {
        if y >= peak_year {
            if v.partial_cmp(&best_val) == Some(std::cmp::Ordering::Less) {
                best_val = v;
                best_idx = i;
            }
        }
    }
    ValueAtYear { value: values[best_idx], year: years[best_idx] }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Task 2: Peak/trough tests ---

    #[test]
    fn peak_of_grow_then_decline() {
        // Parabola: -(year - 1950)^2 + 2500, peaking at year 1950
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();

        let peak = find_peak(&years, &values);
        assert_eq!(peak.year, 1950.0);
        assert_eq!(peak.value, 2500.0);
    }

    #[test]
    fn peak_of_monotonically_increasing() {
        // Linear 0..100, peak at end
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = (0..=100).map(|i| i as f64).collect();

        let peak = find_peak(&years, &values);
        assert_eq!(peak.year, 2000.0);
        assert_eq!(peak.value, 100.0);
    }

    #[test]
    fn trough_after_peak() {
        // Parabola peaking at 1950; trough after 1950 is at 2000 = 0.0
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();

        let trough = find_trough_after_peak(&years, &values, 1950.0);
        assert_eq!(trough.year, 2000.0);
        assert_eq!(trough.value, 0.0);
    }

    #[test]
    fn trough_when_peak_at_end() {
        // Increasing sequence — peak at end, trough after peak = final value
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = (0..=100).map(|i| i as f64).collect();

        let trough = find_trough_after_peak(&years, &values, 2000.0);
        assert_eq!(trough.year, 2000.0);
        assert_eq!(trough.value, 100.0);
    }
}
