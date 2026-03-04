//! Industrial and service capital sector.
//!
//! Capital grows through investment (a fraction of industrial output) and
//! declines through depreciation. As non-renewable resources deplete,
//! the capital-output ratio rises, reducing effective production.
//!
//! Reference year for normalizations: 1970.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Industrial capital output ratio in 1970 [1975 USD capital / 1975 USD output / yr]
const ICOR_1970: f64 = 3.0;
/// Service capital output ratio in 1970
const SCOR_1970: f64 = 1.0;
/// Social adjustment delay [years] — World3-03: SAD = 20 yr.
/// Delayed IOPC drives desired family size (social norms lag behind actual income).
const SOCIAL_ADJUSTMENT_DELAY: f64 = 20.0;

pub struct CapitalDerivatives {
    pub d_industrial_capital: f64,
    pub d_service_capital: f64,
    pub d_perceived_iopc: f64,
}

/// Compute d(industrial_capital)/dt and d(service_capital)/dt.
///
/// Also fills in the derived auxiliary fields on `state.capital`:
/// `industrial_output`, `industrial_output_per_capita`, `service_output_per_capita`.
pub fn capital_derivatives(
    state: &mut WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> CapitalDerivatives {
    let pop = state.population.population.max(1.0);

    // ----- Industrial output -----
    // World3-03: ICOR is constant at 3.0. Resource scarcity feeds back only
    // through FCAOR (capital fraction allocated to resource extraction), not ICOR.
    let icor = ICOR_1970;

    // Technology progress: output per unit capital improves over time
    let tech_years = (state.time - 1970.0).max(0.0);
    let tech_multiplier = (1.0 + params.technology_growth_rate).powf(tech_years);

    // Fraction of industrial capital consumed by resource extraction
    let capital_for_resources = tables
        .capital_fraction_resource_extraction
        .eval(state.resources.fraction_remaining);

    // Effective productive industrial capital
    let productive_capital = state.capital.industrial_capital
        * (1.0 - capital_for_resources.clamp(0.0, 0.95))
        * tech_multiplier;

    let industrial_output = (productive_capital / icor).max(0.0);
    state.capital.industrial_output = industrial_output;

    let iopc = industrial_output / pop;
    state.capital.industrial_output_per_capita = iopc;

    // ----- Service output -----
    let scor = SCOR_1970;
    let service_output = (state.capital.service_capital / scor).max(0.0);
    state.capital.service_output_per_capita = service_output / pop;

    // ----- Allocation fractions -----
    // World3-03: investment is the residual after consumption, services, and agriculture.
    // fioai = 1 - fioac - fioas - fioaa
    // FIOAA uses smoothed FPC / IFPC(IOPC). The smooth FPC is an ODE stock
    // (preserved by from_vec across RK4 stages), while IFPC scales with
    // industrialization to prevent zero-allocation traps at high food levels.
    // At low IOPC (BAU), IFPC ≈ SFPC so this matches the original allocation.
    // At high IOPC (Tech/Stabilized), IFPC rises, keeping food_ratio moderate.
    let ifpc = tables.indicated_food_per_capita.eval(iopc).max(1.0);
    let food_ratio = state.agriculture.food_per_capita_smooth / ifpc;

    let frac_to_agriculture = tables
        .industrial_fraction_to_agriculture
        .eval(food_ratio);

    // World3-03: FIOAS = table(SOPC / ISOPC). ISOPC (indicated SOPC) scales with
    // industrialization. Simplified: normalize SOPC by a reference value (~$200,
    // the 1970 equilibrium SOPC level). Below 1.0 = services inadequate → invest more.
    let sopc_normalized = state.capital.service_output_per_capita / 200.0;
    let frac_to_services = tables
        .industrial_fraction_to_services
        .eval(sopc_normalized);

    let frac_to_consumption = tables
        .consumption_fraction
        .eval(iopc);

    // Investment is the residual — can be squeezed to zero under pressure.
    // Note: the three allocation fractions are independently determined by lookup
    // tables and may sum > 1.0 in collapse scenarios (max: 0.83+0.30+0.40=1.53).
    // When this happens, investment is clamped to zero — the economy over-allocates
    // to consumption, services, and agriculture while starving investment. This
    // differs from World3-03's joint constraint but produces equivalent collapse
    // dynamics since negative investment is impossible in both models.
    let frac_to_investment = (1.0 - frac_to_consumption - frac_to_services - frac_to_agriculture)
        .max(0.0);

    // ----- Industrial capital dynamics -----
    let investment = industrial_output * frac_to_investment;
    let depreciation_i = state.capital.industrial_capital * params.industrial_depreciation_rate;
    let d_industrial = investment - depreciation_i;

    // ----- Service capital dynamics -----
    // Service capital funded by fraction of industrial output allocated to services
    let service_investment = industrial_output * frac_to_services;
    let depreciation_s = state.capital.service_capital * params.service_depreciation_rate;
    let d_service = service_investment - depreciation_s;

    // ----- Perceived IOPC (social adjustment delay) -----
    // World3-03: DIOPC = Smooth(IOPC, SAD) — first-order exponential delay.
    // Social norms lag behind actual income by ~20 years.
    let d_perceived_iopc = (iopc - state.capital.perceived_iopc) / SOCIAL_ADJUSTMENT_DELAY;

    CapitalDerivatives {
        d_industrial_capital: d_industrial,
        d_service_capital: d_service,
        d_perceived_iopc,
    }
}

// REQ: REQ-001
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
        s.agriculture.food_per_capita = 400.0;
        s.agriculture.food_per_capita_smooth = 400.0;
        s.resources.fraction_remaining = 1.0;
        (s, params, tables)
    }

    #[test]
    fn test_residual_investment() {
        let (mut s, params, tables) = setup();
        capital_derivatives(&mut s, &params, &tables);
        // Industrial output should be positive
        assert!(s.capital.industrial_output > 0.0);
        assert!(s.capital.industrial_output_per_capita > 0.0);
    }

    #[test]
    fn test_perceived_iopc_delay() {
        let (mut s, params, tables) = setup();
        // Set perceived_iopc lower than what actual IOPC will be
        s.capital.perceived_iopc = 10.0;
        let d = capital_derivatives(&mut s, &params, &tables);
        // IOPC should be higher than perceived, so d_perceived_iopc > 0
        assert!(d.d_perceived_iopc > 0.0,
            "d_perceived_iopc {} should be positive when perceived < actual",
            d.d_perceived_iopc);
    }

    #[test]
    fn test_technology_multiplier_at_1900() {
        let (mut s, params, tables) = setup();
        // At 1900, time - 1970 < 0 → tech_years = 0 → multiplier = 1.0
        s.time = 1900.0;
        capital_derivatives(&mut s, &params, &tables);
        // IO should equal IC * (1 - FCAOR(1.0)) / ICOR at 1900 with tech=1.0
        let fcaor = tables.capital_fraction_resource_extraction.eval(1.0);
        let expected_io = s.capital.industrial_capital * (1.0 - fcaor.clamp(0.0, 0.95)) / ICOR_1970;
        assert_relative_eq!(s.capital.industrial_output, expected_io, max_relative = 1e-10);
    }

    #[test]
    fn test_technology_multiplier_at_2000() {
        let (mut s, params, tables) = setup();
        s.time = 2000.0;
        capital_derivatives(&mut s, &params, &tables);
        let io_2000 = s.capital.industrial_output;

        let (mut s2, _, _) = setup();
        s2.time = 1900.0;
        capital_derivatives(&mut s2, &params, &tables);
        let io_1900 = s2.capital.industrial_output;

        // tech_multiplier at 2000 = (1 + 0.002)^30
        let expected_ratio = (1.0 + params.technology_growth_rate).powf(30.0);
        assert_relative_eq!(io_2000 / io_1900, expected_ratio, max_relative = 1e-10);
    }

    #[test]
    fn test_resource_depletion_reduces_output() {
        let (mut s1, params, tables) = setup();
        s1.resources.fraction_remaining = 1.0;
        capital_derivatives(&mut s1, &params, &tables);
        let io_full = s1.capital.industrial_output;

        let (mut s2, _, _) = setup();
        s2.resources.fraction_remaining = 0.2;
        capital_derivatives(&mut s2, &params, &tables);
        let io_depleted = s2.capital.industrial_output;

        assert!(io_depleted < io_full,
            "IO with FR=0.2 ({io_depleted}) should be less than FR=1.0 ({io_full})");
    }

    #[test]
    fn test_allocation_fractions_investment_residual() {
        let (mut s, params, tables) = setup();
        let d = capital_derivatives(&mut s, &params, &tables);
        // Investment must be non-negative (residual is clamped to 0)
        // d_industrial = investment - depreciation, investment >= 0
        // So d_industrial >= -depreciation
        let depreciation = s.capital.industrial_capital * params.industrial_depreciation_rate;
        assert!(d.d_industrial_capital >= -depreciation - 1e-10,
            "d_industrial {} should be >= -depreciation {}", d.d_industrial_capital, depreciation);
    }

    #[test]
    fn test_investment_squeezed_to_zero() {
        let (mut s, params, tables) = setup();
        // Extreme low food → high frac_to_agriculture
        // Low service output → high frac_to_services
        // Together with consumption, sum > 1 → investment = 0
        s.agriculture.food_per_capita = 50.0; // far below indicated FPC → high FIOAA
        s.agriculture.food_per_capita_smooth = 50.0; // consistent with raw FPC
        s.capital.service_capital = 1.0; // minimal services
        let d = capital_derivatives(&mut s, &params, &tables);
        // Investment squeezed → d_industrial ≈ -depreciation ≤ 0
        assert!(d.d_industrial_capital <= 0.0,
            "d_industrial {} should be <= 0 when investment is squeezed", d.d_industrial_capital);
    }

    #[test]
    fn test_service_capital_dynamics() {
        let (mut s, params, tables) = setup();
        let d = capital_derivatives(&mut s, &params, &tables);
        // d_service = service_investment - depreciation_s
        let depreciation_s = s.capital.service_capital * params.service_depreciation_rate;
        // Service investment is fraction of IO allocated to services
        // d_service is finite and consistent
        assert!(d.d_service_capital.is_finite());
        // At 1900, services are underdeveloped, so investment > depreciation → growth
        assert!(d.d_service_capital > -depreciation_s,
            "service capital should have positive investment");
    }

    #[test]
    fn test_depreciation_rate_sensitivity() {
        let (mut s1, _, tables) = setup();
        let mut params1 = ScenarioParams::bau();
        params1.industrial_depreciation_rate = 1.0 / 14.0;
        let d1 = capital_derivatives(&mut s1, &params1, &tables);

        let (mut s2, _, _) = setup();
        let mut params2 = ScenarioParams::bau();
        params2.industrial_depreciation_rate = 2.0 / 14.0; // doubled
        let d2 = capital_derivatives(&mut s2, &params2, &tables);

        // Higher depreciation → lower d_industrial_capital
        assert!(d2.d_industrial_capital < d1.d_industrial_capital,
            "doubled depreciation: d_ic={} should be less than d_ic={}",
            d2.d_industrial_capital, d1.d_industrial_capital);
    }

    #[test]
    fn test_capital_auxiliaries_set() {
        let (mut s, params, tables) = setup();
        capital_derivatives(&mut s, &params, &tables);
        assert!(s.capital.industrial_output > 0.0, "IO should be set");
        assert!(s.capital.industrial_output_per_capita > 0.0, "IOPC should be set");
        assert!(s.capital.service_output_per_capita > 0.0, "SOPC should be set");
    }

    #[test]
    fn test_ifpc_rises_with_iopc() {
        // IFPC should increase with IOPC, keeping food_ratio moderate at high industrialization
        let (_, _, tables) = setup();
        let ifpc_low = tables.indicated_food_per_capita.eval(100.0);
        let ifpc_mid = tables.indicated_food_per_capita.eval(600.0);
        let ifpc_high = tables.indicated_food_per_capita.eval(2000.0);
        assert_relative_eq!(ifpc_low, 355.0, max_relative = 0.01); // IFPC(100) = 230 + (100/200)*(480-230)
        assert!(ifpc_mid > ifpc_low, "IFPC should rise with IOPC");
        assert!(ifpc_high > ifpc_mid, "IFPC should continue rising at high IOPC");
        assert!(ifpc_high > 1000.0, "IFPC at IOPC=2000 should exceed 1000");
    }

    #[test]
    fn test_fioaa_floor_prevents_zero_allocation() {
        // With pyworld3-aligned FIOAA table, floor is 0.0 at high food_ratio
        let (_, _, tables) = setup();
        let fioaa_at_3 = tables.industrial_fraction_to_agriculture.eval(3.0);
        let fioaa_at_4 = tables.industrial_fraction_to_agriculture.eval(4.0);
        let fioaa_at_10 = tables.industrial_fraction_to_agriculture.eval(10.0);
        assert!(fioaa_at_3 >= 0.0, "FIOAA should be non-negative at food_ratio=3.0");
        assert!(fioaa_at_3 < 0.1, "FIOAA should be small at high food_ratio");
        assert!(fioaa_at_4 >= 0.0, "FIOAA should be non-negative at food_ratio=4.0");
        assert!(fioaa_at_10 >= 0.0, "FIOAA should be non-negative beyond table range");
    }
}
