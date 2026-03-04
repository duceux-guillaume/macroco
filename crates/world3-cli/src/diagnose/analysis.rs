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
        if y >= peak_year && v.partial_cmp(&best_val) == Some(std::cmp::Ordering::Less) {
            best_val = v;
            best_idx = i;
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

    // Helper to build a Phase from index range
    let make_phase = |kind: &PhaseKind, start: usize, end: usize| -> Phase {
        let start_year = years[start];
        let end_year = years[end];
        let start_value = values[start];
        let end_value = values[end];
        let duration = end_year - start_year;
        let avg_annual_rate = if duration > 0.0 && start_value.abs() > 1e-12 {
            (end_value - start_value) / (start_value * duration)
        } else if duration > 0.0 {
            (end_value - start_value) / duration
        } else {
            0.0
        };
        Phase { kind: kind.clone(), start_year, end_year, start_value, end_value, avg_annual_rate }
    };

    // Merge consecutive same-kind steps into phases
    let mut phases: Vec<Phase> = Vec::new();
    let mut phase_start = 0;

    for i in 1..step_kinds.len() {
        if step_kinds[i] != step_kinds[phase_start] {
            phases.push(make_phase(&step_kinds[phase_start], phase_start, i));
            phase_start = i;
        }
    }

    // Close the last phase
    phases.push(make_phase(&step_kinds[phase_start], phase_start, years.len() - 1));

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

/// Detect anomalies (NaN, Inf, negative values) in a time series.
///
/// Returns a list of `Anomaly` entries for each problematic data point.
pub fn detect_anomalies(name: &str, years: &[f64], values: &[f64]) -> Vec<Anomaly> {
    assert_eq!(years.len(), values.len(), "years and values must have same length");

    let mut anomalies = Vec::new();
    for (i, (&y, &v)) in years.iter().zip(values.iter()).enumerate() {
        if v.is_nan() {
            anomalies.push(Anomaly {
                year: y,
                variable: name.to_string(),
                kind: AnomalyKind::NaN,
                value: v,
            });
        } else if v.is_infinite() {
            anomalies.push(Anomaly {
                year: y,
                variable: name.to_string(),
                kind: AnomalyKind::Inf,
                value: v,
            });
        } else if v < 0.0 {
            anomalies.push(Anomaly {
                year: y,
                variable: name.to_string(),
                kind: AnomalyKind::Negative,
                value: v,
            });
        }
        // Discontinuity: large jump relative to previous value
        if i > 0 && !v.is_nan() && !values[i - 1].is_nan()
            && !v.is_infinite() && !values[i - 1].is_infinite()
        {
            let prev = values[i - 1];
            if prev.abs() > 1e-12 {
                let jump = ((v - prev) / prev).abs();
                if jump > 10.0 {
                    // >1000% change in one step
                    anomalies.push(Anomaly {
                        year: y,
                        variable: name.to_string(),
                        kind: AnomalyKind::Discontinuity,
                        value: v,
                    });
                }
            }
        }
    }
    anomalies
}

/// Find the year-over-year step with the highest positive relative rate of change.
///
/// Returns the rate (as a fraction, e.g. 0.05 = 5%/yr) and the year at which it occurs.
/// If the series is non-increasing, returns a rate of 0 at the first year.
pub fn max_growth_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert_eq!(years.len(), values.len(), "years and values must have same length");
    assert!(years.len() >= 2, "need at least 2 data points");

    let mut best_rate = 0.0_f64;
    let mut best_year = years[0];

    for i in 0..years.len() - 1 {
        let dt = years[i + 1] - years[i];
        if dt <= 0.0 || values[i].abs() < 1e-12 {
            continue;
        }
        let rate = (values[i + 1] - values[i]) / (values[i] * dt);
        if rate > best_rate {
            best_rate = rate;
            best_year = years[i];
        }
    }
    ValueAtYear { value: best_rate, year: best_year }
}

/// Find the year-over-year step with the most negative relative rate of change.
///
/// Returns the rate (as a negative fraction) and the year at which it occurs.
/// If the series is non-decreasing, returns a rate of 0 at the first year.
pub fn max_decline_rate(years: &[f64], values: &[f64]) -> ValueAtYear {
    assert_eq!(years.len(), values.len(), "years and values must have same length");
    assert!(years.len() >= 2, "need at least 2 data points");

    let mut best_rate = 0.0_f64;
    let mut best_year = years[0];

    for i in 0..years.len() - 1 {
        let dt = years[i + 1] - years[i];
        if dt <= 0.0 || values[i].abs() < 1e-12 {
            continue;
        }
        let rate = (values[i + 1] - values[i]) / (values[i] * dt);
        if rate < best_rate {
            best_rate = rate;
            best_year = years[i];
        }
    }
    ValueAtYear { value: best_rate, year: best_year }
}

/// Detect inflection points where the second derivative changes sign.
///
/// Uses second differences of the value series. Returns the year and value
/// at each detected sign change.
pub fn find_inflection_points(years: &[f64], values: &[f64]) -> Vec<ValueAtYear> {
    assert_eq!(years.len(), values.len(), "years and values must have same length");

    if values.len() < 3 {
        return Vec::new();
    }

    // Compute second differences and detect sign changes
    let mut inflections = Vec::new();
    let n = values.len();

    // second_diff[i] = values[i+2] - 2*values[i+1] + values[i]
    let second_diffs: Vec<f64> = (0..n - 2)
        .map(|i| values[i + 2] - 2.0 * values[i + 1] + values[i])
        .collect();

    // Track the last non-zero second difference sign to detect changes
    // that pass through zero (e.g. symmetric sigmoid).
    let mut prev_sign: Option<bool> = None; // true = positive
    let mut prev_nonzero_idx: usize = 0;

    for (i, &sd) in second_diffs.iter().enumerate() {
        if sd == 0.0 {
            // Exact zero — potential inflection point itself.
            // Check if there's a sign change around it.
            if let Some(positive) = prev_sign {
                // Look ahead for the next non-zero second diff
                if let Some(next) = second_diffs[i + 1..].iter().find(|&&s| s != 0.0) {
                    let next_positive = *next > 0.0;
                    if positive != next_positive {
                        // Sign change through zero — inflection at i+1
                        let idx = i + 1;
                        inflections.push(ValueAtYear {
                            value: values[idx],
                            year: years[idx],
                        });
                        prev_sign = Some(next_positive);
                    }
                }
            }
            continue;
        }

        let current_positive = sd > 0.0;
        if let Some(positive) = prev_sign {
            if positive != current_positive {
                // Sign change — inflection between prev_nonzero_idx and i
                // Attribute to the midpoint index
                let idx = (prev_nonzero_idx + i) / 2 + 1;
                inflections.push(ValueAtYear {
                    value: values[idx],
                    year: years[idx],
                });
            }
        }
        prev_sign = Some(current_positive);
        prev_nonzero_idx = i;
    }

    inflections
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
        let values: Vec<f64> = (1900..=2000)
            .map(|y| -(y as f64 - 1950.0).powi(2) + 2500.0)
            .collect();
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

    // --- Task 4: Anomaly detection and rate computation tests ---

    #[test]
    fn anomaly_detects_nan() {
        let years = vec![1900.0, 1901.0, 1902.0];
        let values = vec![1.0, f64::NAN, 3.0];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::NaN);
        assert_eq!(anomalies[0].year, 1901.0);
    }

    #[test]
    fn anomaly_detects_negative() {
        let years = vec![1900.0, 1901.0, 1902.0];
        let values = vec![1.0, -0.5, 3.0];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::Negative);
        assert_eq!(anomalies[0].year, 1901.0);
    }

    #[test]
    fn anomaly_detects_inf() {
        let years = vec![1900.0, 1901.0, 1902.0];
        let values = vec![1.0, f64::INFINITY, 3.0];
        let anomalies = detect_anomalies("test", &years, &values);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::Inf);
        assert_eq!(anomalies[0].year, 1901.0);
    }

    #[test]
    fn no_anomalies_in_clean_series() {
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = (0..=100).map(|i| i as f64).collect();
        let anomalies = detect_anomalies("test", &years, &values);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn max_growth_rate_of_exponential() {
        // Exponential growth: 100 * e^(0.03 * (year - 1900))
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| 100.0 * (0.03 * (y - 1900.0)).exp()).collect();

        let rate = max_growth_rate(&years, &values);
        assert!(rate.value > 0.0, "growth rate should be positive, got {}", rate.value);
    }

    #[test]
    fn max_decline_rate_of_declining() {
        // Exponential decay: 100 * e^(-0.03 * (year - 1900))
        let years: Vec<f64> = (1900..=2000).map(|y| y as f64).collect();
        let values: Vec<f64> = years.iter().map(|&y| 100.0 * (-0.03 * (y - 1900.0)).exp()).collect();

        let rate = max_decline_rate(&years, &values);
        assert!(rate.value < 0.0, "decline rate should be negative, got {}", rate.value);
    }

    #[test]
    fn inflection_point_of_sigmoid() {
        // Sigmoid centered at 2000: 1 / (1 + e^(-0.1 * (year - 2000)))
        // Inflection point should be near year 2000
        let years: Vec<f64> = (1900..=2100).map(|y| y as f64).collect();
        let values: Vec<f64> = years
            .iter()
            .map(|&y| 1.0 / (1.0 + (-0.1 * (y - 2000.0)).exp()))
            .collect();

        let inflections = find_inflection_points(&years, &values);
        assert!(!inflections.is_empty(), "expected at least one inflection point");
        // The inflection point should be near year 2000
        let near_2000 = inflections.iter().any(|ip| (ip.year - 2000.0).abs() < 5.0);
        assert!(near_2000, "expected inflection near 2000, got: {:?}",
            inflections.iter().map(|ip| ip.year).collect::<Vec<_>>());
    }
}
