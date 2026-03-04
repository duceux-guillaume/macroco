//! Agricultural sector.
//!
//! Food production depends on the amount of arable land under cultivation
//! and the yield per hectare. Yield is enhanced by capital inputs (fertilizer,
//! machinery) and degraded by pollution. Arable land stock changes as
//! new land is developed (from potentially-arable reserves) and existing
//! land degrades.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Inherent land fertility [kg / hectare / year] — World3-03: ILF = 600
const INHERENT_LAND_FERTILITY: f64 = 600.0;
/// Total potential arable land area [hectares] (estimate based on FAO)
const TOTAL_POTENTIAL_ARABLE: f64 = 3.2e9;
/// Land development time [years] — delay between investment decision and land available
const LAND_DEVELOPMENT_TIME: f64 = 10.0;
/// Normal land erosion fraction [yr⁻¹]. World3-03: alln = 1000 yr → base = 1/1000 = 0.001
const LAND_EROSION_RATE: f64 = 0.001;
/// Urban-industrial land development time [years] — World3-03: UILD = 10
const UIL_DEVELOPMENT_TIME: f64 = 10.0;

/// Food shortage perception delay [years] — World3-03: FSPD = 2 yr.
/// Smoothed food per capita breaks agriculture-capital allocation oscillation.
const FOOD_SHORTAGE_PERCEPTION_DELAY: f64 = 2.0;

pub struct AgricultureDerivatives {
    pub d_arable_land: f64,
    pub d_potentially_arable_land: f64,
    pub d_urban_industrial_land: f64,
    pub d_land_fertility: f64,
    pub d_food_per_capita_smooth: f64,
}

/// Compute agricultural derivatives and update auxiliary fields on `state.agriculture`.
pub fn agriculture_derivatives(
    state: &mut WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> AgricultureDerivatives {
    let pop = state.population.population.max(1.0);

    // ---- Agricultural inputs per hectare ----
    // Fraction of industrial output allocated to agriculture (food-pressure driven).
    // Uses smoothed food per capita (FSPD=2yr ODE stock) for allocation stability.
    let food_ratio = if params.subsistence_food_per_capita > 0.0 {
        state.agriculture.food_per_capita_smooth / params.subsistence_food_per_capita
    } else {
        1.0
    };
    let frac_to_agri = tables.industrial_fraction_to_agriculture.eval(food_ratio);
    let agri_output_total = state.capital.industrial_output * frac_to_agri;

    let arable = state.agriculture.arable_land.max(1.0);
    let agri_inputs_per_ha = agri_output_total / arable;
    state.agriculture.agricultural_inputs_per_hectare = agri_inputs_per_ha;

    // ---- Land yield ----
    // World3-03: ly = lfert × lymc(aiph) × lymap(ppolx)
    // land_fertility replaces the constant LAND_YIELD_1900 as the base yield
    let yield_multiplier_capital = tables
        .land_yield_multiplier_capital
        .eval(agri_inputs_per_ha);
    let yield_multiplier_pollution = tables
        .land_yield_multiplier_pollution
        .eval(state.pollution.pollution_index);

    let land_fertility = state.agriculture.land_fertility.max(1.0);
    let land_yield = land_fertility
        * yield_multiplier_capital
        * yield_multiplier_pollution
        * params.agricultural_technology;
    state.agriculture.land_yield = land_yield;

    // ---- Food production ----
    let food = arable * land_yield;
    state.agriculture.food = food;
    state.agriculture.food_per_capita = food / pop;

    // ---- Urban-industrial land ----
    // World3-03: UIL is a first-order delay converging to UILPC(IOPC) × POP
    let iopc = state.capital.industrial_output_per_capita;
    let uilpc = tables.urban_industrial_land_per_capita.eval(iopc);
    let uil_desired = uilpc * pop;
    let uil = state.agriculture.urban_industrial_land;
    let d_uil_unconstrained = (uil_desired - uil) / UIL_DEVELOPMENT_TIME;
    // Constrain UIL growth: cannot convert more arable land than available
    let d_uil = if d_uil_unconstrained > 0.0 {
        d_uil_unconstrained.min(arable * 0.1) // max 10% of arable per year
    } else {
        d_uil_unconstrained
    };

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
    let land_yield_ratio = if INHERENT_LAND_FERTILITY > 0.0 {
        land_yield / INHERENT_LAND_FERTILITY
    } else {
        1.0
    };
    let erosion_mult = tables.land_erosion_multiplier.eval(land_yield_ratio);
    let protected_fraction = params.land_protection_fraction.clamp(0.0, 0.5);
    let erosion_rate = arable * LAND_EROSION_RATE * erosion_mult * (1.0 - protected_fraction);

    // UIL growth takes from arable land (only when UIL is expanding)
    let uil_from_arable = d_uil.max(0.0);

    // ---- Land fertility dynamics ----
    // World3-03: lfert' = lfr - lfd
    //   lfd = lfert × LFDR(pollution_index)    — degradation from pollution
    //   lfr = (ILF - lfert) / LFRT(FALM(fr))  — regeneration from maintenance
    let lfdr = tables.land_fertility_degradation.eval(state.pollution.pollution_index);
    let lfd = land_fertility * lfdr;

    let falm = tables.fraction_land_maintenance.eval(food_ratio);
    let lfrt = tables.land_fertility_regeneration_time.eval(falm);
    let lfr = if lfrt > 0.0 {
        (INHERENT_LAND_FERTILITY - land_fertility) / lfrt
    } else {
        0.0
    };

    // ---- Food perception smoothing (FSPD) ----
    // First-order delay: smoothed fpc tracks actual fpc with a 2-year lag.
    let fpc_actual = state.agriculture.food_per_capita;
    let fpc_smooth = state.agriculture.food_per_capita_smooth;
    let d_fpc_smooth = (fpc_actual - fpc_smooth) / FOOD_SHORTAGE_PERCEPTION_DELAY;

    AgricultureDerivatives {
        d_arable_land: land_development_rate - erosion_rate - uil_from_arable,
        d_potentially_arable_land: -land_development_rate,
        d_urban_industrial_land: d_uil,
        d_land_fertility: lfr - lfd,
        d_food_per_capita_smooth: d_fpc_smooth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::lookup::tables::WorldLookupTables;
    use crate::model::params::ScenarioParams;
    use crate::model::state::WorldState;

    fn setup() -> (WorldState, ScenarioParams, WorldLookupTables) {
        let mut s = WorldState::initial_1900();
        let params = ScenarioParams::bau();
        let tables = WorldLookupTables::load();
        // Pre-populate capital auxiliaries
        s.capital.industrial_output = 2.1e11 / 3.0; // IC / ICOR
        s.capital.industrial_output_per_capita = s.capital.industrial_output / 1.6e9;
        s.agriculture.food_per_capita_smooth = 400.0;
        s.pollution.pollution_index = 0.05;
        (s, params, tables)
    }

    #[test]
    fn test_land_fertility_degradation() {
        let (mut s, params, tables) = setup();
        // At high pollution, land fertility should decrease
        s.pollution.pollution_index = 30.0;
        let d = agriculture_derivatives(&mut s, &params, &tables);
        assert!(d.d_land_fertility < 0.0,
            "d_land_fertility {} should be negative at high pollution", d.d_land_fertility);
    }

    #[test]
    fn test_uil_dynamics() {
        let (mut s, params, tables) = setup();
        // With initial UIL at 8.2e6 and moderate IOPC, UIL should grow
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // UIL change direction depends on desired vs current
        // At 1900 with low IOPC, desired UIL ~ UILPC(43.75) * 1.6e9
        // Just check it's finite
        assert!(d.d_urban_industrial_land.is_finite());
    }

    #[test]
    fn test_food_production() {
        let (mut s, params, tables) = setup();
        agriculture_derivatives(&mut s, &params, &tables);
        // food = arable_land × land_yield
        assert!(s.agriculture.food > 0.0, "food should be positive");
        assert!(s.agriculture.food_per_capita > 0.0, "food/cap should be positive");
        // food ≈ arable_land × land_yield
        let expected_food = s.agriculture.arable_land * s.agriculture.land_yield;
        assert!((s.agriculture.food - expected_food).abs() / expected_food < 0.01);
    }

    #[test]
    fn test_food_per_capita_smooth_derivative() {
        let (mut s, params, tables) = setup();
        // Set smooth below actual → derivative should be positive
        s.agriculture.food_per_capita_smooth = 100.0;
        s.agriculture.food_per_capita = 400.0;
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // After agriculture runs, fpc is recomputed. Smooth derivative should push toward actual.
        assert!(d.d_food_per_capita_smooth.is_finite());
    }

    #[test]
    fn test_agricultural_technology_doubles_yield() {
        let (mut s1, _, tables) = setup();
        let mut params1 = ScenarioParams::bau();
        params1.agricultural_technology = 1.0;
        agriculture_derivatives(&mut s1, &params1, &tables);
        let yield1 = s1.agriculture.land_yield;

        let (mut s2, _, _) = setup();
        let mut params2 = ScenarioParams::bau();
        params2.agricultural_technology = 2.0;
        agriculture_derivatives(&mut s2, &params2, &tables);
        let yield2 = s2.agriculture.land_yield;

        assert_relative_eq!(yield2, yield1 * 2.0, max_relative = 1e-10);
    }

    #[test]
    fn test_land_protection_reduces_erosion() {
        let (mut s1, _, tables) = setup();
        let mut params_no = ScenarioParams::bau();
        params_no.land_protection_fraction = 0.0;
        let d1 = agriculture_derivatives(&mut s1, &params_no, &tables);

        let (mut s2, _, _) = setup();
        let mut params_prot = ScenarioParams::bau();
        params_prot.land_protection_fraction = 0.3;
        let d2 = agriculture_derivatives(&mut s2, &params_prot, &tables);

        // Protection reduces erosion → d_arable with protection should be higher
        // (less negative erosion term)
        assert!(d2.d_arable_land > d1.d_arable_land,
            "protected d_arable ({}) should exceed unprotected ({})",
            d2.d_arable_land, d1.d_arable_land);
    }

    #[test]
    fn test_zero_industrial_output_still_produces_food() {
        let (mut s, params, tables) = setup();
        s.capital.industrial_output = 0.0;
        s.capital.industrial_output_per_capita = 0.0;
        agriculture_derivatives(&mut s, &params, &tables);
        // Food comes from land fertility even without industrial inputs
        assert!(s.agriculture.food > 0.0,
            "food should be positive even with zero industrial output");
    }

    #[test]
    fn test_no_potentially_arable_land_stops_development() {
        let (mut s, params, tables) = setup();
        s.agriculture.potentially_arable_land = 0.0;
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // No PAL → land_development_rate = 0
        // d_potentially_arable should be 0 (can't develop what doesn't exist)
        assert_relative_eq!(d.d_potentially_arable_land, 0.0, epsilon = 1e-15);
    }

    #[test]
    fn test_uil_constrained_by_arable_land() {
        let (mut s, params, tables) = setup();
        // High IOPC → large desired UIL, but constrained by 10% of arable
        s.capital.industrial_output_per_capita = 5000.0;
        s.agriculture.arable_land = 1000.0; // small arable land
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // d_uil should be at most arable * 0.1 = 100
        assert!(d.d_urban_industrial_land <= 1000.0 * 0.1 + 1e-10,
            "d_uil {} should be <= 100 (10% of arable)", d.d_urban_industrial_land);
    }

    #[test]
    fn test_food_per_capita_smooth_converges() {
        let (mut s, params, tables) = setup();
        // Set smooth well below what fpc will actually be
        s.agriculture.food_per_capita_smooth = 50.0;
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // After computing actual fpc (which will be > 50 from land fertility),
        // smooth should be pulled toward actual → positive derivative
        assert!(d.d_food_per_capita_smooth > 0.0,
            "d_fpc_smooth {} should be positive when smooth < actual",
            d.d_food_per_capita_smooth);
    }

    #[test]
    fn test_yield_components_multiply() {
        let (mut s, params, tables) = setup();
        agriculture_derivatives(&mut s, &params, &tables);
        // land_yield = fertility × LYMC × LYMAP × tech
        let fertility = s.agriculture.land_fertility.max(1.0);
        let lymc = tables.land_yield_multiplier_capital.eval(s.agriculture.agricultural_inputs_per_hectare);
        let lymap = tables.land_yield_multiplier_pollution.eval(s.pollution.pollution_index);
        let expected = fertility * lymc * lymap * params.agricultural_technology;
        assert_relative_eq!(s.agriculture.land_yield, expected, max_relative = 1e-10);
    }

    #[test]
    fn test_land_development_and_erosion_balance() {
        let (mut s, params, tables) = setup();
        let d = agriculture_derivatives(&mut s, &params, &tables);
        // d_arable = development - erosion - uil_from_arable
        // d_potentially_arable = -development
        // So development = -d_potentially_arable
        let development = -d.d_potentially_arable_land;
        // uil_from_arable = max(d_uil, 0)
        let uil_from_arable = d.d_urban_industrial_land.max(0.0);
        // erosion = development - d_arable - uil_from_arable
        let erosion = development - d.d_arable_land - uil_from_arable;
        assert!(erosion >= 0.0, "erosion {erosion} should be non-negative");
        assert!(development >= 0.0, "development {development} should be non-negative");
    }
}
