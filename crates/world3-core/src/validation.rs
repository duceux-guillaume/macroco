//! BAU validation against World3 reference dynamics (Meadows 1972/2004).

use crate::model::state::WorldState;
use crate::output::SimulationOutput;

/// Result of a single validation check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

/// Find the peak (max value and its year) of a field across simulation states.
fn find_peak(sim: &SimulationOutput, extract: fn(&WorldState) -> f64) -> (f64, f64) {
    sim.states
        .iter()
        .fold((0.0_f64, 0.0_f64), |(max_val, max_year), s| {
            let v = extract(s);
            if v > max_val {
                (v, s.time)
            } else {
                (max_val, max_year)
            }
        })
}

/// Run all BAU qualitative validation checks against a simulation output.
pub fn validate_bau(sim: &SimulationOutput) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // Population trajectory checkpoints
    for (year, lo, hi) in [
        (1900.0, 1.4e9, 1.8e9),
        (1950.0, 2.0e9, 4.0e9),
        (1970.0, 3.0e9, 5.5e9),
    ] {
        let pop = sim
            .state_at_year(year)
            .map(|s| s.population.population)
            .unwrap_or(f64::NAN);
        let passed = (lo..=hi).contains(&pop);
        results.push(CheckResult {
            label: format!("{:.0} population", year),
            passed,
            detail: format!("{:.3e} [expected {:.1e}–{:.1e}]", pop, lo, hi),
        });
    }

    // Population peak
    let (peak_pop, peak_year) = find_peak(sim, |s| s.population.population);
    let peak_ok = (5.0e9..=12.0e9).contains(&peak_pop) && (1990.0..=2080.0).contains(&peak_year);
    results.push(CheckResult {
        label: "Population peak".into(),
        passed: peak_ok,
        detail: format!(
            "{:.2e} at year {:.0} [expected 5B–12B, 1990–2080]",
            peak_pop, peak_year
        ),
    });

    // Population decline after peak
    let pop_2100 = sim
        .state_at_year(2100.0)
        .map(|s| s.population.population)
        .unwrap_or(f64::NAN);
    let decline_ok = pop_2100 < peak_pop * 0.95;
    results.push(CheckResult {
        label: "Population decline after peak".into(),
        passed: decline_ok,
        detail: format!("{:.2e} at 2100 vs peak {:.2e}", pop_2100, peak_pop),
    });

    // NNR fraction checkpoints
    for (year, lo, hi) in [(2000.0, 0.0, 0.60), (2100.0, 0.0, 0.30)] {
        let nnr = sim
            .state_at_year(year)
            .map(|s| s.resources.fraction_remaining)
            .unwrap_or(f64::NAN);
        let passed = (lo..=hi).contains(&nnr);
        results.push(CheckResult {
            label: format!("{:.0} NNR fraction", year),
            passed,
            detail: format!("{:.4} [expected {:.2}–{:.2}]", nnr, lo, hi),
        });
    }

    // NNR monotonic — fail if any checkpoint year is missing
    let checkpoints = [
        1920.0, 1940.0, 1960.0, 1980.0, 2000.0, 2020.0, 2040.0, 2060.0, 2080.0, 2100.0,
    ];
    let nnr_values: Vec<Option<f64>> = checkpoints
        .iter()
        .map(|&y| sim.state_at_year(y).map(|s| s.resources.fraction_remaining))
        .collect();
    let any_missing = nnr_values.iter().any(|v| v.is_none());
    let nnr_monotonic = if any_missing {
        false
    } else {
        nnr_values.windows(2).all(|pair| {
            let a = pair[0].unwrap();
            let b = pair[1].unwrap();
            b <= a + 0.001
        })
    };
    let detail = if any_missing {
        "Missing checkpoint year(s) in simulation".into()
    } else if nnr_monotonic {
        "OK".into()
    } else {
        "Non-monotonic NNR detected".into()
    };
    results.push(CheckResult {
        label: "NNR monotonically decreasing".into(),
        passed: nnr_monotonic,
        detail,
    });

    // Pollution peak
    let (max_pollution, _) = find_peak(sim, |s| s.pollution.pollution_index);
    let poll_ok = (1.0..=100.0).contains(&max_pollution);
    results.push(CheckResult {
        label: "Peak pollution index".into(),
        passed: poll_ok,
        detail: format!("{:.2} [expected 1.0–100.0]", max_pollution),
    });

    // IOPC collapse
    let (peak_iopc, peak_iopc_year) =
        find_peak(sim, |s| s.capital.industrial_output_per_capita);
    let iopc_2100 = sim
        .state_at_year(2100.0)
        .map(|s| s.capital.industrial_output_per_capita)
        .unwrap_or(f64::NAN);
    let iopc_ok = iopc_2100 <= peak_iopc * 0.5;
    results.push(CheckResult {
        label: "IOPC collapse".into(),
        passed: iopc_ok,
        detail: format!(
            "{:.0} at 2100, peak {:.0} at {:.0} [expected <50% of peak]",
            iopc_2100, peak_iopc, peak_iopc_year
        ),
    });

    // Life expectancy peak (skip initial year before recomputation)
    let (peak_le, peak_le_year) = sim
        .states
        .iter()
        .filter(|s| s.time >= 1910.0)
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.life_expectancy > mp {
                (s.population.life_expectancy, s.time)
            } else {
                (mp, my)
            }
        });
    let le_peak_ok = (45.0..=80.0).contains(&peak_le);
    results.push(CheckResult {
        label: "Peak life expectancy".into(),
        passed: le_peak_ok,
        detail: format!(
            "{:.1} yr at {:.0} [expected 45–80]",
            peak_le, peak_le_year
        ),
    });

    // Life expectancy decline
    let le_2100 = sim
        .state_at_year(2100.0)
        .map(|s| s.population.life_expectancy)
        .unwrap_or(f64::NAN);
    let le_decline_ok = le_2100 <= peak_le * 0.8;
    results.push(CheckResult {
        label: "Life expectancy decline".into(),
        passed: le_decline_ok,
        detail: format!(
            "{:.1} at 2100 vs peak {:.1} at {:.0}",
            le_2100, peak_le, peak_le_year
        ),
    });

    results
}
