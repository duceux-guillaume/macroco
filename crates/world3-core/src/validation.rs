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
    // Upper bound widened to 2090 because Delay3 perceived-LE adds ~20yr
    // demographic inertia vs the original Delay1 model.
    let peak_ok = (5.0e9..=16.0e9).contains(&peak_pop) && (2020.0..=2090.0).contains(&peak_year);
    results.push(CheckResult {
        label: "Population peak".into(),
        passed: peak_ok,
        detail: format!(
            "{:.2e} at year {:.0} [expected 5B–16B, 2020–2090]",
            peak_pop, peak_year
        ),
    });

    // Population decline after peak
    let pop_2100 = sim
        .state_at_year(2100.0)
        .map(|s| s.population.population)
        .unwrap_or(f64::NAN);
    // Relaxed from 95% to 97%: Delay3 perceived-LE creates more gradual
    // population decline because compensatory fertility responds slowly.
    let decline_ok = pop_2100 < peak_pop * 0.97;
    results.push(CheckResult {
        label: "Population decline after peak".into(),
        passed: decline_ok,
        detail: format!("{:.2e} at 2100 vs peak {:.2e}", pop_2100, peak_pop),
    });

    // NNR fraction checkpoints
    // NNR at 2000 widened to 0.70 after LE calibration (resource_efficiency=1.05
    // slows depletion). 2100 tightened to match main's <0.25 threshold.
    for (year, lo, hi) in [(2000.0, 0.0, 0.70), (2100.0, 0.0, 0.25)] {
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
    let iopc_peak_year_ok = (1990.0..=2060.0).contains(&peak_iopc_year);
    results.push(CheckResult {
        label: "IOPC peak year".into(),
        passed: iopc_peak_year_ok,
        detail: format!("{:.0} [expected 1990–2060]", peak_iopc_year),
    });

    let iopc_ok = iopc_2100 <= peak_iopc * 0.5;
    results.push(CheckResult {
        label: "IOPC collapse".into(),
        passed: iopc_ok,
        detail: format!(
            "{:.0} at 2100, peak {:.0} at {:.0} [expected <50% of peak]",
            iopc_2100, peak_iopc, peak_iopc_year
        ),
    });

    // Food per capita: peak then decline
    let (peak_fpc, peak_fpc_year) = find_peak(sim, |s| s.agriculture.food_per_capita);
    results.push(CheckResult {
        label: "Food/capita peak range".into(),
        passed: peak_fpc >= 500.0 && peak_fpc <= 1200.0,
        detail: format!("peak={:.0} kg/yr (expected 500–1200)", peak_fpc),
    });
    results.push(CheckResult {
        label: "Food/capita peak timing".into(),
        passed: peak_fpc_year >= 2000.0 && peak_fpc_year <= 2070.0,
        detail: format!("peak year={:.0} (expected 2000–2070)", peak_fpc_year),
    });
    let fpc_2100 = sim.state_at_year(2100.0).map(|s| s.agriculture.food_per_capita).unwrap_or(0.0);
    results.push(CheckResult {
        label: "Food/capita collapse by 2100".into(),
        passed: fpc_2100 < peak_fpc * 0.6,
        detail: format!("2100={:.0}, peak={:.0}, ratio={:.2} (need <0.60)", fpc_2100, peak_fpc, fpc_2100 / peak_fpc),
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
