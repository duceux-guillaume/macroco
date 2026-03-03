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
    // Capital-output ratio rises as resources deplete
    let cor_multiplier = tables
        .capital_output_ratio_resources
        .eval(state.resources.fraction_remaining);

    // Effective ICOR (higher = less output per unit capital)
    let icor = ICOR_1970 * cor_multiplier;

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
    let food_ratio = if params.subsistence_food_per_capita > 0.0 {
        state.agriculture.food_per_capita / params.subsistence_food_per_capita
    } else {
        1.0
    };

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

    // Investment is the residual — can be squeezed to near zero under pressure
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
