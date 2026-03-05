// REQ: REQ-001
//! BAU Qualitative Dynamics Tests
//!
//! Ensures the BAU scenario produces the Limits to Growth overshoot-and-collapse
//! pattern without constraining specific years from the 1972 study.

mod common;
use common::bau_sim;

/// BAU population must peak between 2020-2090 then decline.
/// Upper bound widened from 2080 to 2090 because Delay3 perceived-LE
/// adds ~20yr demographic inertia vs the original Delay1 model.
#[test]
fn bau_population_peaks_then_declines() {
    let sim = bau_sim();
    let (peak_pop, peak_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.population > mp { (s.population.population, s.time) } else { (mp, my) }
        });

    assert!(peak_pop >= 5.0e9, "Population peak {:.2e} should be >= 5B", peak_pop);
    assert!(peak_pop <= 16.0e9, "Population peak {:.2e} should be <= 16B", peak_pop);
    assert!(peak_year >= 2020.0, "Population peak year {:.0} should be >= 2020", peak_year);
    assert!(peak_year <= 2090.0, "Population peak year {:.0} should be <= 2090", peak_year);

    let pop_2100 = sim.state_at_year(2100.0).expect("missing year 2100").population.population;
    // Relaxed from 95% to 97%: Delay3 perceived-LE creates more gradual
    // population decline because compensatory fertility responds slowly.
    assert!(pop_2100 < peak_pop * 0.97,
        "2100 pop {:.2e} should be < 97% of peak {:.2e} (decline)", pop_2100, peak_pop);
}

/// BAU IOPC must peak then collapse (2100 IOPC < 50% of peak).
#[test]
fn bau_iopc_peaks_then_collapses() {
    let sim = bau_sim();
    let (peak_iopc, peak_year) = sim.states.iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.capital.industrial_output_per_capita > mp {
                (s.capital.industrial_output_per_capita, s.time)
            } else { (mp, my) }
        });

    assert!(peak_year >= 1990.0, "IOPC peak year {:.0} should be >= 1990", peak_year);
    assert!(peak_year <= 2060.0, "IOPC peak year {:.0} should be <= 2060", peak_year);

    let iopc_2100 = sim.state_at_year(2100.0).expect("missing year 2100").capital.industrial_output_per_capita;
    assert!(iopc_2100 < peak_iopc * 0.5,
        "2100 IOPC {:.0} should be < 50% of peak {:.0} (collapse)", iopc_2100, peak_iopc);
}

/// BAU NNR must monotonically decline to < 25% by 2100.
#[test]
fn bau_nnr_monotonic_depletion() {
    let sim = bau_sim();
    let nnr_2100 = sim.state_at_year(2100.0).expect("missing year 2100").resources.fraction_remaining;
    assert!(nnr_2100 < 0.25,
        "2100 NNR fraction {:.3} should be < 0.25", nnr_2100);

    let monotonic = [1920.0, 1940.0, 1960.0, 1980.0, 2000.0, 2020.0, 2040.0, 2060.0, 2080.0, 2100.0]
        .windows(2)
        .all(|pair| {
            let a = sim.state_at_year(pair[0]).expect("missing NNR state").resources.fraction_remaining;
            let b = sim.state_at_year(pair[1]).expect("missing NNR state").resources.fraction_remaining;
            b <= a + 0.001
        });
    assert!(monotonic, "NNR fraction should decrease monotonically");
}

/// BAU life expectancy must peak (45-80yr) then decline.
#[test]
fn bau_life_expectancy_peaks_then_declines() {
    let sim = bau_sim();
    let (peak_le, _peak_year) = sim.states.iter()
        .filter(|s| s.time >= 1910.0)
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.life_expectancy > mp {
                (s.population.life_expectancy, s.time)
            } else { (mp, my) }
        });

    assert!(peak_le >= 45.0, "Peak LE {:.1} should be >= 45", peak_le);
    assert!(peak_le <= 80.0, "Peak LE {:.1} should be <= 80", peak_le);

    let le_2100 = sim.state_at_year(2100.0).expect("missing year 2100").population.life_expectancy;
    assert!(le_2100 < peak_le * 0.8,
        "2100 LE {:.1} should be < 80% of peak {:.1} (decline)", le_2100, peak_le);
}

/// BAU pollution must peak above 1.0 (above 1970 baseline level).
#[test]
fn bau_pollution_peaks_above_baseline() {
    let sim = bau_sim();
    let max_pollution = sim.states.iter()
        .map(|s| s.pollution.pollution_index)
        .fold(0.0_f64, f64::max);
    assert!(max_pollution > 1.0,
        "Peak pollution {:.2} should exceed 1.0 (1970 baseline)", max_pollution);
}
