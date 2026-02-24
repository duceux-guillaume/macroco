//! Population sector.
//!
//! Tracks four age cohorts: 0–14, 15–44, 45–64, 65+.
//! Births enter cohort 0–14; deaths occur in all cohorts; aging moves people
//! between cohorts. The sector depends heavily on food, health services,
//! crowding, and pollution through lookup-table multipliers on life expectancy.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Base life expectancy [years] before applying lookup-table multipliers.
///
/// With BASE = 20 and the new lem_health table (which returns < 1.0 at low sopc):
///   1900: 20 × 1.49 × 0.76 × 1.41 × 1.0 ≈ 32 yr  (lem_health < 1 = poor health services)
///   1970: 20 × 1.50 × 1.37 × 1.30 × 1.0 ≈ 53 yr  (lem_health ≈ 1.37 = adequate services)
const LIFE_EXPECTANCY_BASE: f64 = 20.0;

/// Cohort durations (years spent in each cohort before aging out)
const COHORT_0_14_DURATION: f64 = 15.0;
const COHORT_15_44_DURATION: f64 = 30.0;
const COHORT_45_64_DURATION: f64 = 20.0;

pub struct PopulationDerivatives {
    pub d_cohort_0_14: f64,
    pub d_cohort_15_44: f64,
    pub d_cohort_45_64: f64,
    pub d_cohort_65_plus: f64,
}

/// Compute population derivatives and update auxiliary fields:
/// `life_expectancy`, `birth_rate`, `death_rate`, `fertility_rate`.
pub fn population_derivatives(
    state: &mut WorldState,
    params: &ScenarioParams,
    tables: &WorldLookupTables,
) -> PopulationDerivatives {
    let pop = state.population.population.max(1.0);

    // ---- Life expectancy ----
    let food_ratio = state.agriculture.food_per_capita / params.subsistence_food_per_capita;

    // Effective health services per capita [1975 USD/person/yr].
    // The lem_health table x-axis is 0..100 USD/person/yr, calibrated so that
    // 1970 conditions (~50 USD/person/yr) give LMH ≈ 1.62 multiplier.
    let health_services = state.capital.service_output_per_capita
        * params.health_investment_multiplier;

    // Crowding ratio: loosely based on population relative to 1970
    let crowding_ratio = pop / 3.6e9;

    let lem_food = tables.life_exp_multiplier_food.eval(food_ratio);
    let lem_health = tables.life_exp_multiplier_health.eval(health_services);
    let lem_crowding = tables.life_exp_multiplier_crowding.eval(crowding_ratio);
    let lem_pollution = tables
        .life_exp_multiplier_pollution
        .eval(state.pollution.pollution_index);

    let life_expectancy = LIFE_EXPECTANCY_BASE
        * lem_food
        * lem_health
        * lem_crowding
        * lem_pollution;
    state.population.life_expectancy = life_expectancy.clamp(5.0, 85.0);

    // ---- Fertility / birth rate ----
    // Desired family size decreases with industrial output per capita
    let iopc = state.capital.industrial_output_per_capita;
    let desired_family_size = tables.desired_family_size.eval(iopc);

    // Family planning ramps in from zero at 1900 to full efficacy by family_planning_year
    let fp_ramp = if params.family_planning_year <= 1900.0 {
        1.0
    } else {
        ((state.time - 1900.0) / (params.family_planning_year - 1900.0)).clamp(0.0, 1.0)
    };
    let fp_effectiveness = params.family_planning_efficacy * fp_ramp;
    let fp_multiplier = tables.family_planning_multiplier.eval(fp_effectiveness);

    // Food effect on fertility
    let food_fertility = tables.food_fertility_multiplier.eval(food_ratio);

    let total_fertility_rate = desired_family_size * fp_multiplier * food_fertility;
    state.population.fertility_rate = total_fertility_rate.clamp(0.5, 8.0);

    // Births = fertile-age women × TFR / reproductive period
    // Women aged 15–44 represent ~half the cohort and are fertile
    let fertile_women = state.population.cohort_15_44 * 0.5;
    let births_per_year = fertile_women * total_fertility_rate / COHORT_15_44_DURATION;
    let birth_rate = births_per_year / pop;
    state.population.birth_rate = birth_rate;

    // ---- Age-cohort mortality ----
    // Base annual death fraction = 1 / life_expectancy.
    // Cohort multipliers calibrated so weighted average ≈ 0.9 at 1900 age structure
    // (high child mortality and small 65+ cohort), yielding crude death rate ~2.8%
    // at LE=32 — consistent with historical 1900 world average.
    let base_mort = 1.0 / life_expectancy.max(1.0);

    let deaths_0_14 = state.population.cohort_0_14 * base_mort * 0.8;
    let deaths_15_44 = state.population.cohort_15_44 * base_mort * 0.5;
    let deaths_45_64 = state.population.cohort_45_64 * base_mort * 1.0;
    let deaths_65_plus = state.population.cohort_65_plus * base_mort * 3.0;

    let total_deaths = deaths_0_14 + deaths_15_44 + deaths_45_64 + deaths_65_plus;
    state.population.death_rate = total_deaths / pop;

    // ---- Cohort aging rates ----
    // People age out of each cohort after spending the cohort duration in it
    let aging_0_to_15 = state.population.cohort_0_14 / COHORT_0_14_DURATION;
    let aging_15_to_45 = state.population.cohort_15_44 / COHORT_15_44_DURATION;
    let aging_45_to_65 = state.population.cohort_45_64 / COHORT_45_64_DURATION;

    PopulationDerivatives {
        d_cohort_0_14: births_per_year - aging_0_to_15 - deaths_0_14,
        d_cohort_15_44: aging_0_to_15 - aging_15_to_45 - deaths_15_44,
        d_cohort_45_64: aging_15_to_45 - aging_45_to_65 - deaths_45_64,
        d_cohort_65_plus: aging_45_to_65 - deaths_65_plus,
    }
}
