//! Non-renewable resource sector.
//!
//! Resources are depleted by industrial activity. As the fraction remaining
//! falls, the cost multiplier rises, diverting ever-larger shares of industrial
//! capital to resource extraction instead of productive output.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Resource depletion coefficient [NNR_fraction / (person × USD/person/yr × year)].
///
/// World3-03: NRI = 1e12 resource units, NRUR at 1970 ≈ 1e10 units/yr.
/// Calibrated against historical NNR depletion data (NNR RMSE% = 9.5%).
/// At 0.3e-14, Collapse produces NNR fraction ~0.11 by 2100, consistent with
/// Meadows 1972 standard run trajectory.
const RESOURCE_DEPLETION_COEFF: f64 = 0.3e-14;

/// Compute the rate of change of non-renewable resources.
///
/// Returns `d(nonrenewable_resources)/dt` in resource units per year.
/// This is always negative (resources are consumed, never replenished).
///
/// Mechanism:
/// - Per-capita resource use scales with industrial output per capita (IOPC).
/// - Effective efficiency = resource_efficiency × (1 + growth_rate)^max(year-1970, 0).
/// - Total extraction = POP × per_capita_use / effective_efficiency.
/// - The FCAOR table (capital fraction for resource extraction) is used ONLY by
///   the capital sector to reduce productive output — it does NOT appear here.
pub fn resource_derivative(
    state: &WorldState,
    params: &ScenarioParams,
    _tables: &WorldLookupTables,
) -> f64 {
    let pop = state.population.population;
    if pop <= 0.0 {
        return 0.0;
    }

    // Per-capita resource demand scales with industrial output per capita
    let iopc = state.capital.industrial_output_per_capita.max(0.0);

    // Effective resource efficiency grows over time from 1970 onward,
    // representing improving extraction technology (EOR, horizontal drilling,
    // heap leaching, etc.). Structure mirrors agricultural_technology_growth_rate.
    let eff_years = (state.time - 1970.0).max(0.0);
    let effective_efficiency = params.resource_efficiency
        * (1.0 + params.resource_efficiency_growth_rate).powf(eff_years);

    // Extraction rate = POP × IOPC × coefficient / efficiency
    // Decreases naturally as resources deplete (via feedback through capital output)
    let extraction_rate = pop * iopc * RESOURCE_DEPLETION_COEFF / effective_efficiency;

    -extraction_rate
}

/// Compute auxiliary variables for the resource sector.
///
/// Updates `state.resources.fraction_remaining` in place.
/// Must be called before capital sector uses the cost multiplier.
pub fn compute_resource_auxiliaries(state: &mut WorldState, _tables: &WorldLookupTables) {
    state.resources.fraction_remaining =
        state.resources.nonrenewable_resources.clamp(0.0, 1.0);

    // World3-03: resource scarcity feeds back ONLY through FCAOR (capital fraction
    // allocated to resource extraction in capital.rs), not through ICOR. ICOR is constant.
}

// REQ: REQ-001
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::lookup::tables::WorldLookupTables;
    use crate::model::params::ScenarioParams;
    use crate::model::state::WorldState;

    #[test]
    fn test_resource_derivative_known_value() {
        let mut s = WorldState::initial_1900();
        // Set known IOPC auxiliary
        s.capital.industrial_output_per_capita = 43.75;
        let params = ScenarioParams::collapse();
        let tables = WorldLookupTables::load();
        let d = resource_derivative(&s, &params, &tables);
        // pop=1.6e9, iopc=43.75, coeff=0.3e-14
        // At 1900 (before 1970): growth_rate has no effect, effective_efficiency = resource_efficiency = 1.0
        let effective_eff = params.resource_efficiency;
        let expected = -(1.6e9 * 43.75 * RESOURCE_DEPLETION_COEFF / effective_eff);
        assert_relative_eq!(d, expected, epsilon = 1e-10);
    }

    #[test]
    fn test_resource_derivative_zero_population() {
        let mut s = WorldState::initial_1900();
        s.population.population = 0.0;
        s.capital.industrial_output_per_capita = 100.0;
        let params = ScenarioParams::collapse();
        let tables = WorldLookupTables::load();
        let d = resource_derivative(&s, &params, &tables);
        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_resource_derivative_negative_iopc_clamped() {
        let mut s = WorldState::initial_1900();
        s.capital.industrial_output_per_capita = -500.0;
        let params = ScenarioParams::collapse();
        let tables = WorldLookupTables::load();
        let d = resource_derivative(&s, &params, &tables);
        // Negative IOPC clamped to 0 → extraction = 0 → derivative = 0
        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_resource_derivative_efficiency_halves_rate() {
        let mut s = WorldState::initial_1900();
        s.capital.industrial_output_per_capita = 200.0;
        let tables = WorldLookupTables::load();

        let mut params1 = ScenarioParams::collapse();
        params1.resource_efficiency = 1.0;
        let d1 = resource_derivative(&s, &params1, &tables);

        let mut params2 = ScenarioParams::collapse();
        params2.resource_efficiency = 2.0;
        let d2 = resource_derivative(&s, &params2, &tables);

        // Double efficiency → half depletion rate
        assert_relative_eq!(d2, d1 / 2.0, epsilon = 1e-15);
    }

    #[test]
    fn test_resource_derivative_always_nonpositive() {
        let tables = WorldLookupTables::load();
        let params = ScenarioParams::collapse();
        // Test across various populations and IOPC values
        for &pop in &[1e6, 1e9, 1e10] {
            for &iopc in &[0.0, 100.0, 1000.0, 10000.0] {
                let mut s = WorldState::initial_1900();
                s.population.population = pop;
                s.capital.industrial_output_per_capita = iopc;
                let d = resource_derivative(&s, &params, &tables);
                assert!(d <= 0.0, "derivative should be <= 0 for pop={pop}, iopc={iopc}, got {d}");
            }
        }
    }

    #[test]
    fn test_resource_efficiency_growth_rate() {
        let mut s = WorldState::initial_1900();
        s.capital.industrial_output_per_capita = 200.0;
        let tables = WorldLookupTables::load();

        // At 1960 (before 1970): growth rate has no effect
        s.time = 1960.0;
        let mut params = ScenarioParams::collapse();
        let d_pre = resource_derivative(&s, &params, &tables);

        // Verify: same as if growth_rate were 0
        params.resource_efficiency_growth_rate = 0.0;
        let d_no_growth = resource_derivative(&s, &params, &tables);
        assert_relative_eq!(d_pre, d_no_growth, epsilon = 1e-15);

        // At 2020 (50 years after 1970): growth rate reduces depletion
        s.time = 2020.0;
        let mut params_with = ScenarioParams::collapse();
        let d_2020_with = resource_derivative(&s, &params_with, &tables);

        params_with.resource_efficiency_growth_rate = 0.0;
        let d_2020_without = resource_derivative(&s, &params_with, &tables);

        // With growth rate, depletion should be slower (less negative)
        assert!(d_2020_with > d_2020_without,
            "growth rate should reduce depletion: with={d_2020_with}, without={d_2020_without}");

        // Check magnitude: at 0.0035/yr for 50 years, efficiency grows by (1.0035)^50 = 1.191
        let expected_ratio = (1.0 + 0.0035_f64).powf(50.0);
        assert_relative_eq!(d_2020_without / d_2020_with, expected_ratio, max_relative = 1e-10);
    }

    #[test]
    fn test_compute_resource_auxiliaries_clamp_high() {
        let mut s = WorldState::initial_1900();
        s.resources.nonrenewable_resources = 1.5;
        let tables = WorldLookupTables::load();
        compute_resource_auxiliaries(&mut s, &tables);
        assert_eq!(s.resources.fraction_remaining, 1.0);
    }

    #[test]
    fn test_compute_resource_auxiliaries_clamp_low() {
        let mut s = WorldState::initial_1900();
        s.resources.nonrenewable_resources = -0.5;
        let tables = WorldLookupTables::load();
        compute_resource_auxiliaries(&mut s, &tables);
        assert_eq!(s.resources.fraction_remaining, 0.0);
    }
}
