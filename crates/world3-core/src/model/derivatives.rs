//! Top-level derivative function: `dy/dt = f(t, y, params)`.
//!
//! This function computes the rate of change for all 10 ODE stocks.
//! Sector computation order is fixed to satisfy dependencies:
//!
//!   1. Resources (other sectors need fraction_remaining for cost multiplier)
//!   2. Capital    (depends on resource fraction; produces industrial_output)
//!   3. Agriculture (depends on industrial_output for inputs; depends on pollution)
//!   4. Population  (depends on food, services, pollution)
//!   5. Pollution   (depends on industrial_output, agricultural_inputs)
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
/// Auxiliary fields on the returned state are zeroed — only the 10 ODE
/// stocks (cohorts, capitals, arable land, resources, pollution) carry data.
pub fn derivatives(
    state: &WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> WorldState {
    // Work on a mutable copy so sectors can fill in auxiliary fields
    let mut s = state.clone();

    // --- Step 1: Resources ---
    // (Must run first; capital sector needs fraction_remaining)
    resources::compute_resource_auxiliaries(&mut s, tables);
    let d_nnr = resources::resource_derivative(&s, params, tables);

    // --- Step 2: Capital ---
    let cap_deriv = capital::capital_derivatives(&mut s, params, tables);

    // --- Step 3: Agriculture ---
    // (food_per_capita is needed by population and must be current)
    let agri_deriv = agriculture::agriculture_derivatives(&mut s, params, tables);

    // --- Step 4: Pollution ---
    // (pollution_index must be updated before population uses it)
    let d_pollution = pollution::pollution_derivative(&mut s, params, tables);

    // --- Step 5: Population ---
    let pop_deriv = population::population_derivatives(&mut s, params, tables);

    // --- Build derivative state ---
    let mut d = WorldState::zero_derivative(state.time);

    d.population.cohort_0_14 = pop_deriv.d_cohort_0_14;
    d.population.cohort_15_44 = pop_deriv.d_cohort_15_44;
    d.population.cohort_45_64 = pop_deriv.d_cohort_45_64;
    d.population.cohort_65_plus = pop_deriv.d_cohort_65_plus;

    d.capital.industrial_capital = cap_deriv.d_industrial_capital;
    d.capital.service_capital = cap_deriv.d_service_capital;

    d.agriculture.arable_land = agri_deriv.d_arable_land;
    d.agriculture.potentially_arable_land = agri_deriv.d_potentially_arable_land;

    d.resources.nonrenewable_resources = d_nnr;

    d.pollution.persistent_pollution = d_pollution;

    d
}
