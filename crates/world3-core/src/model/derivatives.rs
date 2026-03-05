//! Top-level derivative function: `dy/dt = f(t, y, params)`.
//!
//! This function computes the rate of change for all 20 ODE stocks.
//! Sector computation order is fixed to satisfy dependencies:
//!
//!   0. Pre-seed food_per_capita from stocks (auxiliary, zeroed by from_vec)
//!   1. Resource auxiliaries (fraction_remaining for cost multiplier)
//!   2. Capital    (depends on resource fraction; produces industrial_output)
//!   3. Resource depletion (needs IOPC from capital)
//!   4. Agriculture (depends on industrial_output for inputs; depends on pollution)
//!   5. Pollution   (depends on industrial_output, agricultural_inputs)
//!   6. Population  (depends on food, services, pollution)
//!
//! The function takes a `&WorldState` (read-only) and produces a new
//! `WorldState` that represents the derivatives (stocks are rates of change).
//! Auxiliary fields are computed on a mutable working copy, not exposed outside.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{
    params::ScenarioParams,
    sectors::{agriculture, capital, pollution, population, resources},
    state::WorldState,
};

/// Compute `dy/dt` for the full World 3 state vector.
///
/// Returns a `WorldState` where all stock fields hold *rates of change*
/// (units: [stock_unit / year]), not values. The `time` field is unused.
///
/// Auxiliary fields on the returned state are zeroed — only the 20 ODE
/// stocks (cohorts, perceived_le + 2 stages, capitals, land, fpc_smooth, resources, pollution + 2 stages) carry data.
pub fn derivatives(
    state: &WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> WorldState {
    // Work on a mutable copy so sectors can fill in auxiliary fields
    let mut s = state.clone();

    // --- Step 0: Pre-seed food_per_capita from stocks ---
    // food_per_capita is auxiliary (zeroed by from_vec()); pre-seed with a base-yield
    // estimate so population sector gets a reasonable food_ratio.
    // food_per_capita_smooth is a proper ODE stock (preserved by from_vec/to_vec),
    // so it does NOT need pre-seeding here.
    if s.agriculture.food_per_capita <= 0.0 {
        let pop = s.population.population.max(1.0);
        let base_yield = s.agriculture.land_fertility.max(1.0);
        s.agriculture.food_per_capita = s.agriculture.arable_land * base_yield / pop;
    }

    // --- Step 1: Resource auxiliaries ---
    // (Capital sector needs fraction_remaining for cost multiplier and FCAOR)
    resources::compute_resource_auxiliaries(&mut s, tables);

    // --- Step 2: Capital ---
    // (Produces industrial_output, needed by agriculture and resource depletion)
    let cap_deriv = capital::capital_derivatives(&mut s, params, tables);

    // --- Step 3: Resource depletion ---
    // (Needs IOPC from capital — must run AFTER capital_derivatives)
    let d_nnr = resources::resource_derivative(&s, params, tables);

    // --- Step 4: Agriculture ---
    // (food_per_capita is needed by population and must be current)
    let agri_deriv = agriculture::agriculture_derivatives(&mut s, params, tables);

    // --- Step 5: Pollution ---
    // (pollution_index must be updated before population uses it)
    let (d_pollution, d_poll_stage1, d_poll_stage2, d_poll_stage3) =
        pollution::pollution_derivatives(&mut s, params, tables);

    // --- Step 6: Population ---
    let pop_deriv = population::population_derivatives(&mut s, params, tables);

    // --- Build derivative state ---
    let mut d = WorldState::zero_derivative(state.time);

    d.population.cohort_0_14 = pop_deriv.d_cohort_0_14;
    d.population.cohort_15_44 = pop_deriv.d_cohort_15_44;
    d.population.cohort_45_64 = pop_deriv.d_cohort_45_64;
    d.population.cohort_65_plus = pop_deriv.d_cohort_65_plus;
    d.population.perceived_le = pop_deriv.d_perceived_le;
    d.population.perceived_le_stage1 = pop_deriv.d_perceived_le_stage1;
    d.population.perceived_le_stage2 = pop_deriv.d_perceived_le_stage2;

    d.capital.industrial_capital = cap_deriv.d_industrial_capital;
    d.capital.service_capital = cap_deriv.d_service_capital;
    d.capital.perceived_iopc = cap_deriv.d_perceived_iopc;

    d.agriculture.arable_land = agri_deriv.d_arable_land;
    d.agriculture.potentially_arable_land = agri_deriv.d_potentially_arable_land;
    d.agriculture.urban_industrial_land = agri_deriv.d_urban_industrial_land;
    d.agriculture.land_fertility = agri_deriv.d_land_fertility;
    d.agriculture.food_per_capita_smooth = agri_deriv.d_food_per_capita_smooth;

    d.resources.nonrenewable_resources = d_nnr;

    d.pollution.persistent_pollution = d_pollution;
    d.pollution.pollution_appearance_stage1 = d_poll_stage1;
    d.pollution.pollution_appearance_stage2 = d_poll_stage2;
    d.pollution.pollution_appearance_buffer = d_poll_stage3;

    d
}

// REQ: REQ-001
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn setup() -> (WorldState, ScenarioParams, WorldLookupTables) {
        let s = WorldState::initial_1900();
        let params = ScenarioParams::bau();
        let tables = WorldLookupTables::load();
        (s, params, tables)
    }

    #[test]
    fn test_returns_20_stock_derivatives() {
        let (s, params, tables) = setup();
        let d = derivatives(&s, &params, &tables);
        let v = d.to_vec();
        assert_eq!(v.len(), 20);
        // Most should be nonzero at 1900
        let nonzero_count = v.iter().filter(|x| x.abs() > 1e-20).count();
        assert!(nonzero_count >= 12,
            "expected >= 12 nonzero derivatives at 1900, got {nonzero_count}");
    }

    #[test]
    fn test_does_not_mutate_input() {
        let (s, params, tables) = setup();
        let s_before = s.to_vec();
        let _ = derivatives(&s, &params, &tables);
        let s_after = s.to_vec();
        for (i, (a, b)) in s_before.iter().zip(s_after.iter()).enumerate() {
            assert_eq!(*a, *b, "stock {i} was mutated by derivatives()");
        }
    }

    #[test]
    fn test_food_per_capita_preseeding() {
        // from_vec zeroes FPC auxiliary; derivatives should still work
        let (s, params, tables) = setup();
        let v = s.to_vec();
        let s2 = WorldState::from_vec(1900.0, &v);
        // food_per_capita is zeroed by from_vec
        assert_eq!(s2.agriculture.food_per_capita, 0.0);
        let d = derivatives(&s2, &params, &tables);
        let dv = d.to_vec();
        for (i, val) in dv.iter().enumerate() {
            assert!(val.is_finite(), "derivative stock {i} is not finite after preseeding");
        }
    }

    #[test]
    fn test_sector_ordering_consistency() {
        let (s, params, tables) = setup();
        let d = derivatives(&s, &params, &tables);
        // Resources deplete at 1900
        assert!(d.resources.nonrenewable_resources < 0.0,
            "d_nnr {} should be negative (depletion)", d.resources.nonrenewable_resources);
        // Total population grows at 1900
        let net_pop = d.population.cohort_0_14 + d.population.cohort_15_44
            + d.population.cohort_45_64 + d.population.cohort_65_plus;
        assert!(net_pop > 0.0,
            "net population change {} should be positive at 1900", net_pop);
        // Industrial capital grows at 1900
        assert!(d.capital.industrial_capital > 0.0,
            "d_industrial_capital {} should be positive at 1900", d.capital.industrial_capital);
    }

    #[test]
    fn test_zero_state_does_not_panic() {
        let s = WorldState::default();
        let params = ScenarioParams::bau();
        let tables = WorldLookupTables::load();
        let d = derivatives(&s, &params, &tables);
        let dv = d.to_vec();
        for (i, val) in dv.iter().enumerate() {
            assert!(val.is_finite(), "derivative stock {i} is not finite for zero state");
        }
    }

    #[test]
    fn test_auxiliary_fields_zeroed_in_output() {
        let (s, params, tables) = setup();
        let d = derivatives(&s, &params, &tables);
        // Auxiliary fields should be zero in derivative output
        assert_eq!(d.population.population, 0.0);
        assert_eq!(d.population.birth_rate, 0.0);
        assert_eq!(d.population.death_rate, 0.0);
        assert_eq!(d.population.life_expectancy, 0.0);
        assert_eq!(d.capital.industrial_output, 0.0);
        assert_eq!(d.capital.industrial_output_per_capita, 0.0);
        assert_eq!(d.agriculture.food, 0.0);
        assert_eq!(d.agriculture.food_per_capita, 0.0);
        assert_eq!(d.pollution.pollution_index, 0.0);
    }

    #[test]
    fn test_depleted_resources_reduces_capital_growth() {
        let (s1, params, tables) = setup();
        let d1 = derivatives(&s1, &params, &tables);

        let mut s2 = WorldState::initial_1900();
        s2.resources.nonrenewable_resources = 0.05; // nearly depleted
        s2.resources.fraction_remaining = 0.05;
        let d2 = derivatives(&s2, &params, &tables);

        assert!(d2.capital.industrial_capital < d1.capital.industrial_capital,
            "d_ic with depleted NNR ({}) should be less than full ({})",
            d2.capital.industrial_capital, d1.capital.industrial_capital);
    }

    #[test]
    fn test_symmetry_with_individual_sectors() {
        let (s, params, tables) = setup();
        // Run full derivatives
        let d_full = derivatives(&s, &params, &tables);

        // Run sectors individually in the same order
        let mut s_manual = s.clone();
        if s_manual.agriculture.food_per_capita <= 0.0 {
            let pop = s_manual.population.population.max(1.0);
            let base_yield = s_manual.agriculture.land_fertility.max(1.0);
            s_manual.agriculture.food_per_capita = s_manual.agriculture.arable_land * base_yield / pop;
        }
        resources::compute_resource_auxiliaries(&mut s_manual, &tables);
        let cap = capital::capital_derivatives(&mut s_manual, &params, &tables);
        let d_nnr = resources::resource_derivative(&s_manual, &params, &tables);
        let agri = agriculture::agriculture_derivatives(&mut s_manual, &params, &tables);
        let (d_poll, d_poll_s1, d_poll_s2, d_poll_s3) = pollution::pollution_derivatives(&mut s_manual, &params, &tables);
        let pop_d = population::population_derivatives(&mut s_manual, &params, &tables);

        // Compare
        assert_relative_eq!(d_full.capital.industrial_capital, cap.d_industrial_capital, epsilon = 1e-10);
        assert_relative_eq!(d_full.capital.service_capital, cap.d_service_capital, epsilon = 1e-10);
        assert_relative_eq!(d_full.resources.nonrenewable_resources, d_nnr, epsilon = 1e-10);
        assert_relative_eq!(d_full.agriculture.arable_land, agri.d_arable_land, epsilon = 1e-10);
        assert_relative_eq!(d_full.pollution.persistent_pollution, d_poll, epsilon = 1e-10);
        assert_relative_eq!(d_full.pollution.pollution_appearance_stage1, d_poll_s1, epsilon = 1e-10);
        assert_relative_eq!(d_full.pollution.pollution_appearance_stage2, d_poll_s2, epsilon = 1e-10);
        assert_relative_eq!(d_full.pollution.pollution_appearance_buffer, d_poll_s3, epsilon = 1e-10);
        assert_relative_eq!(d_full.population.cohort_0_14, pop_d.d_cohort_0_14, epsilon = 1e-10);
        assert_relative_eq!(d_full.population.cohort_65_plus, pop_d.d_cohort_65_plus, epsilon = 1e-10);
    }
}
