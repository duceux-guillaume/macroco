//! Population sector.
//!
//! Tracks four age cohorts: 0–14, 15–44, 45–64, 65+.
//! Births enter cohort 0–14; deaths occur in all cohorts; aging moves people
//! between cohorts. The sector depends heavily on food, health services,
//! crowding, and pollution through lookup-table multipliers on life expectancy.
//!
//! Mortality uses World3-03 M1-M4 lookup tables (cohort-specific rates indexed
//! by life expectancy) rather than fixed multipliers on 1/LE.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Base life expectancy [years] before applying lookup-table multipliers.
/// World3-03: LEN = 28 years (subsistence-level LE).
const LIFE_EXPECTANCY_BASE: f64 = 28.0;

/// Cohort durations (years spent in each cohort before aging out)
const COHORT_0_14_DURATION: f64 = 15.0;
const COHORT_15_44_DURATION: f64 = 30.0;
const COHORT_45_64_DURATION: f64 = 20.0;

/// Maximum total fertility — biological maximum [children / woman]
/// World3-03: MTF = 12
const MAX_TOTAL_FERTILITY: f64 = 12.0;

/// Lifetime perception delay [years] — World3-03: LPD = 20
const LIFETIME_PERCEPTION_DELAY: f64 = 20.0;

pub struct PopulationDerivatives {
    pub d_cohort_0_14: f64,
    pub d_cohort_15_44: f64,
    pub d_cohort_45_64: f64,
    pub d_cohort_65_plus: f64,
    pub d_perceived_le: f64,
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

    // Effective health services per capita.
    // In World3-03, EHSPC = delayed(HSAPC) where HSAPC = SOPC × health_fraction.
    // Simplified: use fraction of service output allocated to health.
    let health_fraction = tables.fraction_services_health.eval(
        state.capital.service_output_per_capita / 100.0, // normalize
    );
    let health_services = state.capital.service_output_per_capita
        * health_fraction
        * params.health_investment_multiplier;

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
    state.population.life_expectancy = life_expectancy.clamp(5.0, 90.0);

    // ---- Fertility / birth rate ----
    // World3-03: desired family size is driven by DIOPC (delayed IOPC, 20-year
    // social adjustment lag). This prevents instantaneous demographic transition
    // when income rises rapidly.
    let perceived_iopc = state.capital.perceived_iopc;
    let desired_family_size = tables.desired_family_size.eval(perceived_iopc);

    // Compensatory fertility (CMPLE): when perceived LE is low (high infant
    // mortality), women have more children to compensate for expected deaths.
    // Uses perceived_le (20-year delay) so fertility expectations lag reality.
    let perceived_le = state.population.perceived_le.max(5.0);
    let cmple = tables.compensatory_fertility.eval(perceived_le);

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

    // Desired fertility including all social/compensatory effects
    let desired_fertility = desired_family_size * cmple * fp_multiplier * food_fertility;

    // Fecundity multiplier (FM): biological ceiling on fertility from health.
    // At low LE, malnutrition and disease reduce maximum achievable fertility.
    // World3-03: TF = min(MTF × FM(LE), desired_fertility)
    let fm = tables.fecundity_multiplier.eval(life_expectancy);
    let biological_max = MAX_TOTAL_FERTILITY * fm;
    let total_fertility_rate = desired_fertility.min(biological_max);

    state.population.fertility_rate = total_fertility_rate.clamp(0.5, 8.0);

    // Births = fertile-age women × TFR / reproductive period
    let fertile_women = state.population.cohort_15_44 * 0.5;
    let births_per_year = fertile_women * total_fertility_rate / COHORT_15_44_DURATION;
    let birth_rate = births_per_year / pop;
    state.population.birth_rate = birth_rate;

    // ---- Age-cohort mortality (World3-03 M1-M4 tables) ----
    // Each table gives the annual mortality rate for a cohort as a function of LE.
    let m1 = tables.mortality_0_14.eval(life_expectancy);
    let m2 = tables.mortality_15_44.eval(life_expectancy);
    let m3 = tables.mortality_45_64.eval(life_expectancy);
    let m4 = tables.mortality_65_plus.eval(life_expectancy);

    let deaths_0_14 = state.population.cohort_0_14 * m1;
    let deaths_15_44 = state.population.cohort_15_44 * m2;
    let deaths_45_64 = state.population.cohort_45_64 * m3;
    let deaths_65_plus = state.population.cohort_65_plus * m4;

    let total_deaths = deaths_0_14 + deaths_15_44 + deaths_45_64 + deaths_65_plus;
    state.population.death_rate = total_deaths / pop;

    // ---- Perceived life expectancy (20-year delay) ----
    // World3-03: PLE = Delay3(LE, LPD=20yr). Simplified as first-order delay.
    let d_perceived_le = (life_expectancy - perceived_le) / LIFETIME_PERCEPTION_DELAY;

    // ---- Cohort aging rates ----
    let aging_0_to_15 = state.population.cohort_0_14 / COHORT_0_14_DURATION;
    let aging_15_to_45 = state.population.cohort_15_44 / COHORT_15_44_DURATION;
    let aging_45_to_65 = state.population.cohort_45_64 / COHORT_45_64_DURATION;

    PopulationDerivatives {
        d_cohort_0_14: births_per_year - aging_0_to_15 - deaths_0_14,
        d_cohort_15_44: aging_0_to_15 - aging_15_to_45 - deaths_15_44,
        d_cohort_45_64: aging_15_to_45 - aging_45_to_65 - deaths_45_64,
        d_cohort_65_plus: aging_45_to_65 - deaths_65_plus,
        d_perceived_le,
    }
}
