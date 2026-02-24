//! Agricultural sector.
//!
//! Food production depends on the amount of arable land under cultivation
//! and the yield per hectare. Yield is enhanced by capital inputs (fertilizer,
//! machinery) and degraded by pollution. Arable land stock changes as
//! new land is developed (from potentially-arable reserves) and existing
//! land degrades.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Base land yield in 1900 [kg / hectare / year]
const LAND_YIELD_1900: f64 = 600.0;
/// Total potential arable land area [hectares] (estimate based on FAO)
const TOTAL_POTENTIAL_ARABLE: f64 = 3.2e9;
/// Land development time [years] — delay between investment decision and land available
const LAND_DEVELOPMENT_TIME: f64 = 10.0;
/// Normal land erosion fraction [yr⁻¹] — fraction of land that degrades under normal use
const LAND_EROSION_RATE: f64 = 0.002;

pub struct AgricultureDerivatives {
    pub d_arable_land: f64,
    pub d_potentially_arable_land: f64,
}

/// Compute agricultural derivatives and update auxiliary fields on `state.agriculture`.
pub fn agriculture_derivatives(
    state: &mut WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> AgricultureDerivatives {
    let pop = state.population.population.max(1.0);

    // ---- Agricultural inputs per hectare ----
    // Fraction of industrial output allocated to agriculture (food-pressure driven)
    let food_ratio = if params.subsistence_food_per_capita > 0.0 {
        state.agriculture.food_per_capita / params.subsistence_food_per_capita
    } else {
        1.0
    };
    let frac_to_agri = tables.industrial_fraction_to_agriculture.eval(food_ratio);
    let agri_output_total = state.capital.industrial_output * frac_to_agri;

    let arable = state.agriculture.arable_land.max(1.0);
    let agri_inputs_per_ha = agri_output_total / arable;
    state.agriculture.agricultural_inputs_per_hectare = agri_inputs_per_ha;

    // ---- Land yield ----
    let yield_multiplier_capital = tables
        .land_yield_multiplier_capital
        .eval(agri_inputs_per_ha);
    let yield_multiplier_pollution = tables
        .land_yield_multiplier_pollution
        .eval(state.pollution.pollution_index);

    let land_yield = LAND_YIELD_1900
        * yield_multiplier_capital
        * yield_multiplier_pollution
        * params.agricultural_technology;
    state.agriculture.land_yield = land_yield;

    // ---- Food production ----
    let food = arable * land_yield;
    state.agriculture.food = food;
    state.agriculture.food_per_capita = food / pop;

    // ---- Land development ----
    // New land is developed when food pressure is high and potentially-arable land exists
    let potentially_arable = state.agriculture.potentially_arable_land.max(0.0);

    // Development cost rises as better land is used up
    let land_fraction_developed =
        1.0 - potentially_arable / TOTAL_POTENTIAL_ARABLE.max(1.0);
    let dev_cost_multiplier = tables
        .land_development_cost
        .eval(land_fraction_developed.clamp(0.0, 1.0));

    // Food pressure: if food < subsistence, invest more in land development
    let land_development_desired =
        (state.capital.industrial_output * frac_to_agri * 0.1)
        / dev_cost_multiplier.max(1.0);

    let land_development_rate =
        (land_development_desired / LAND_DEVELOPMENT_TIME).min(potentially_arable / LAND_DEVELOPMENT_TIME);

    // ---- Land erosion / degradation ----
    let land_yield_ratio = if LAND_YIELD_1900 > 0.0 {
        land_yield / LAND_YIELD_1900
    } else {
        1.0
    };
    let erosion_mult = tables.land_erosion_multiplier.eval(land_yield_ratio);
    let protected_fraction = params.land_protection_fraction.clamp(0.0, 0.5);
    let erosion_rate = arable * LAND_EROSION_RATE * erosion_mult * (1.0 - protected_fraction);

    AgricultureDerivatives {
        d_arable_land: land_development_rate - erosion_rate,
        d_potentially_arable_land: -land_development_rate,
    }
}
