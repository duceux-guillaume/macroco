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

/// Relative rate threshold below which a year-to-year change is classified as Plateau.
const PLATEAU_THRESHOLD: f64 = 0.001; // 0.1%/yr

/// Segment a time series into contiguous phases of Growing, Declining, or Plateau.
///
/// Each year-to-year step is classified by its relative rate of change vs `PLATEAU_THRESHOLD`.
/// Consecutive steps of the same kind are merged into a single `Phase`.
/// Panics if inputs are empty or of different lengths.
pub fn segment_phases(years: &[f64], values: &[f64]) -> Vec<Phase> {
    assert_eq!(years.len(), values.len(), "years and values must have same length");
    assert!(years.len() >= 2, "need at least 2 data points for phase segmentation");

    // Classify each step
    let mut step_kinds: Vec<PhaseKind> = Vec::with_capacity(years.len() - 1);
    for i in 0..years.len() - 1 {
        let dt = years[i + 1] - years[i];
        if dt == 0.0 {
            step_kinds.push(PhaseKind::Plateau);
            continue;
        }
        let dv = values[i + 1] - values[i];
        // Use absolute rate when value is near zero to avoid division by zero
        let rate = if values[i].abs() > 1e-12 {
            (dv / values[i]) / dt
        } else {
            dv / dt
        };
        if rate > PLATEAU_THRESHOLD {
            step_kinds.push(PhaseKind::Growing);
        } else if rate < -PLATEAU_THRESHOLD {
            step_kinds.push(PhaseKind::Declining);
        } else {
            step_kinds.push(PhaseKind::Plateau);
        }
    }

    // Merge consecutive same-kind steps into phases
    let mut phases: Vec<Phase> = Vec::new();
    let mut phase_start = 0;

    for i in 1..step_kinds.len() {
        if step_kinds[i] != step_kinds[phase_start] {
            // Close previous phase (covers years[phase_start]..=years[i])
            let start_year = years[phase_start];
            let end_year = years[i];
            let start_value = values[phase_start];
            let end_value = values[i];
            let duration = end_year - start_year;
            let avg_annual_rate = if duration > 0.0 && start_value.abs() > 1e-12 {
                (end_value - start_value) / (start_value * duration)
            } else if duration > 0.0 {
                (end_value - start_value) / duration
            } else {
                0.0
            };
            phases.push(Phase {
                kind: step_kinds[phase_start].clone(),
                start_year,
                end_year,
                start_value,
                end_value,
                avg_annual_rate,
            });
            phase_start = i;
        }
    }

    // Close the last phase (covers years[phase_start]..=years[last])
    let last = years.len() - 1;
    let start_year = years[phase_start];
    let end_year = years[last];
    let start_value = values[phase_start];
    let end_value = values[last];
    let duration = end_year - start_year;
    let avg_annual_rate = if duration > 0.0 && start_value.abs() > 1e-12 {
        (end_value - start_value) / (start_value * duration)
    } else if duration > 0.0 {
        (end_value - start_value) / duration
    } else {
        0.0
    };
    phases.push(Phase {
        kind: step_kinds[phase_start].clone(),
        start_year,
        end_year,
        start_value,
        end_value,
        avg_annual_rate,
    });

    phases
}

/// Check whether a series is monotonically non-decreasing or non-increasing.
pub fn is_monotonic(values: &[f64]) -> bool {
    if values.len() < 2 {
        return true;
    }
    let non_decreasing = values.windows(2).all(|w| w[1] >= w[0]);
    let non_increasing = values.windows(2).all(|w| w[1] <= w[0]);
    non_decreasing || non_increasing
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

    // --- Task 3: Phase segmentation tests ---

    #[test]
    fn phases_grow_then_decline() {
        // Parabola: grows 1900-~1949, plateau near peak, declines ~1951-2000
        // Near the peak the rate drops below PLATEAU_THRESHOLD, producing a
        // small Plateau phase — this is correct behavior.
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();

        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 3, "expected 3 phases (grow/plateau/decline), got {}", phases.len());
        assert_eq!(phases[0].kind, PhaseKind::Growing);
        assert_eq!(phases[1].kind, PhaseKind::Plateau);
        assert_eq!(phases[2].kind, PhaseKind::Declining);
        assert_eq!(phases[0].start_year, 1900.0);
        assert_eq!(phases[2].end_year, 2000.0);
    }

    #[test]
    fn phases_monotonic_decline() {
        // Linear decline from 100 to 0
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = (0..=100).rev().map(|i| i as f64).collect();

        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].kind, PhaseKind::Declining);
    }

    #[test]
    fn monotonic_true_for_declining() {
        let values: Vec<f64> = (0..=100).rev().map(|i| i as f64).collect();
        assert!(is_monotonic(&values));
    }

    #[test]
    fn monotonic_false_for_up_then_down() {
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| -(y - 1950.0).powi(2) + 2500.0).collect();
        assert!(!is_monotonic(&values));
    }

    #[test]
    fn phases_have_avg_annual_rate() {
        // Linear growth: 0 to 100 over 100 years → rate = 1/yr
        // Relative rate = (v2 - v1) / v1 per year, but for avg_annual_rate
        // we use (end_value - start_value) / (start_value * duration)
        // For this series starting at 0, we use absolute rate: (end - start) / duration
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = (0..=100).map(|i| i as f64).collect();

        let phases = segment_phases(&years, &values);
        assert_eq!(phases.len(), 1);
        // avg_annual_rate for a Growing phase should be positive
        assert!(phases[0].avg_annual_rate > 0.0, "expected positive rate");
    }
}
