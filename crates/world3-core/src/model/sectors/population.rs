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

/// Health services impact delay [years] — World3-03: HSID = 20
const HEALTH_SERVICES_IMPACT_DELAY: f64 = 20.0;

pub struct PopulationDerivatives {
    pub d_cohort_0_14: f64,
    pub d_cohort_15_44: f64,
    pub d_cohort_45_64: f64,
    pub d_cohort_65_plus: f64,
    pub d_perceived_le: f64,
    pub d_perceived_le_stage1: f64,
    pub d_perceived_le_stage2: f64,
    pub d_ehspc: f64,
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

    // World3-03: HSAPC table maps SOPC → health spending per capita directly.
    let hsapc = tables.health_services_per_capita.eval(
        state.capital.service_output_per_capita,
    ) * params.health_investment_multiplier;
    // EHSPC = smooth(HSAPC, HSID=20yr) — first-order exponential delay
    let d_ehspc = (hsapc - state.population.ehspc) / HEALTH_SERVICES_IMPACT_DELAY;

    let lem_food = tables.life_exp_multiplier_food.eval(food_ratio);
    // Use smoothed EHSPC (not raw HSAPC) for life expectancy
    let lem_health = tables.life_exp_multiplier_health.eval(state.population.ehspc);

    // World3-03: LMC = 1 - CMI(IOPC) × FPU(POP)
    let cmi = tables.crowding_multiplier_ind.eval(state.capital.industrial_output_per_capita);
    let fpu = tables.fraction_population_urban.eval(pop);
    let lem_crowding = (1.0 - cmi * fpu).max(0.0);
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

    // ---- Perceived life expectancy (Delay3: 3 cascaded first-order stages) ----
    // World3-03: PLE = Delay3(LE, LPD=20yr). Three stages with τ = LPD/3.
    // This gives pipeline-like behavior (more uniform transit time than Delay1).
    let tau = LIFETIME_PERCEPTION_DELAY / 3.0;
    let d_perceived_le_stage1 = (life_expectancy - state.population.perceived_le_stage1) / tau;
    let d_perceived_le_stage2 = (state.population.perceived_le_stage1 - state.population.perceived_le_stage2) / tau;
    let d_perceived_le = (state.population.perceived_le_stage2 - perceived_le) / tau;

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
        d_perceived_le_stage1,
        d_perceived_le_stage2,
        d_ehspc,
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
        // Pre-populate auxiliary fields needed by population
        s.agriculture.food_per_capita = 400.0;
        s.agriculture.food_per_capita_smooth = 400.0;
        s.capital.industrial_output_per_capita = 43.75;
        s.capital.service_output_per_capita = 20.0;
        s.pollution.pollution_index = 0.05;
        (s, params, tables)
    }

    #[test]
    fn test_population_derivatives_1900() {
        let (mut s, params, tables) = setup();
        let d = population_derivatives(&mut s, &params, &tables);
        // At 1900: birth_rate > death_rate (population growing)
        assert!(s.population.birth_rate > s.population.death_rate,
            "birth_rate {} should exceed death_rate {} at 1900",
            s.population.birth_rate, s.population.death_rate);
        // Life expectancy should be in 28-40 range at 1900
        assert!(s.population.life_expectancy >= 28.0 && s.population.life_expectancy <= 45.0,
            "LE {} outside expected 1900 range", s.population.life_expectancy);
        // Net population growth positive
        let net = d.d_cohort_0_14 + d.d_cohort_15_44 + d.d_cohort_45_64 + d.d_cohort_65_plus;
        assert!(net > 0.0, "net population change should be positive at 1900");
    }

    #[test]
    fn test_mortality_tables_used() {
        let (mut s, params, tables) = setup();
        population_derivatives(&mut s, &params, &tables);
        // Death rate should be reasonable (not 0, not 1)
        assert!(s.population.death_rate > 0.001 && s.population.death_rate < 0.1,
            "death_rate {} seems unreasonable", s.population.death_rate);
    }

    #[test]
    fn test_perceived_le_delay() {
        let (mut s, params, tables) = setup();
        // Set perceived_le lower than actual computed LE
        s.population.perceived_le = 20.0;
        let d = population_derivatives(&mut s, &params, &tables);
        // LE should be > perceived_le, so d_perceived_le > 0 (converging up)
        assert!(d.d_perceived_le > 0.0,
            "d_perceived_le {} should be positive when perceived_le < actual LE",
            d.d_perceived_le);
    }

    #[test]
    fn test_life_expectancy_components() {
        let (mut s, params, tables) = setup();
        population_derivatives(&mut s, &params, &tables);
        // LE = 28 × LEM_food × LEM_health × LEM_crowding × LEM_pollution
        let food_ratio = s.agriculture.food_per_capita / params.subsistence_food_per_capita;
        // EHSPC (smoothed) is the input to LMHS, not raw HSAPC
        let cmi = tables.crowding_multiplier_ind.eval(s.capital.industrial_output_per_capita);
        let fpu = tables.fraction_population_urban.eval(s.population.population.max(1.0));
        let lem_crowding = (1.0 - cmi * fpu).max(0.0);
        let lem_food = tables.life_exp_multiplier_food.eval(food_ratio);
        let lem_health = tables.life_exp_multiplier_health.eval(s.population.ehspc);
        let lem_pollution = tables.life_exp_multiplier_pollution.eval(s.pollution.pollution_index);
        let expected = (LIFE_EXPECTANCY_BASE * lem_food * lem_health * lem_crowding * lem_pollution)
            .clamp(5.0, 90.0);
        assert_relative_eq!(s.population.life_expectancy, expected, max_relative = 1e-10);
    }

    #[test]
    fn test_starvation_crashes_life_expectancy() {
        let (mut s, params, tables) = setup();
        s.agriculture.food_per_capita = 50.0; // severe starvation
        population_derivatives(&mut s, &params, &tables);
        // Life expectancy should be near floor
        assert!(s.population.life_expectancy <= 15.0,
            "LE {} should be near floor at fpc=50", s.population.life_expectancy);
    }

    #[test]
    fn test_high_pollution_reduces_life_expectancy() {
        let (mut s_clean, params, tables) = setup();
        s_clean.pollution.pollution_index = 0.05;
        population_derivatives(&mut s_clean, &params, &tables);
        let le_clean = s_clean.population.life_expectancy;

        let (mut s_dirty, _, _) = setup();
        s_dirty.pollution.pollution_index = 50.0;
        population_derivatives(&mut s_dirty, &params, &tables);
        let le_dirty = s_dirty.population.life_expectancy;

        assert!(le_dirty < le_clean,
            "LE at high pollution ({le_dirty}) should be less than clean ({le_clean})");
    }

    #[test]
    fn test_fertility_rate_clamping() {
        let (mut s, params, tables) = setup();
        population_derivatives(&mut s, &params, &tables);
        assert!(s.population.fertility_rate >= 0.5,
            "fertility {} should be >= 0.5", s.population.fertility_rate);
        assert!(s.population.fertility_rate <= 8.0,
            "fertility {} should be <= 8.0", s.population.fertility_rate);
    }

    #[test]
    fn test_family_planning_reduces_fertility() {
        let (mut s1, _, tables) = setup();
        let mut params_no_fp = ScenarioParams::bau();
        params_no_fp.family_planning_efficacy = 0.0;
        s1.time = 2020.0;
        population_derivatives(&mut s1, &params_no_fp, &tables);
        let fert_no_fp = s1.population.fertility_rate;

        let (mut s2, _, _) = setup();
        let mut params_fp = ScenarioParams::bau();
        params_fp.family_planning_efficacy = 0.95;
        params_fp.family_planning_year = 1975.0;
        s2.time = 2020.0;
        population_derivatives(&mut s2, &params_fp, &tables);
        let fert_fp = s2.population.fertility_rate;

        assert!(fert_fp < fert_no_fp,
            "fertility with FP ({fert_fp}) should be less than without ({fert_no_fp})");
    }

    #[test]
    fn test_cohort_aging_rates() {
        let (mut s, params, tables) = setup();
        let d = population_derivatives(&mut s, &params, &tables);
        // aging_0_to_15 = cohort_0_14 / 15
        let aging_0 = s.population.cohort_0_14 / COHORT_0_14_DURATION;
        // aging_15_to_45 = cohort_15_44 / 30
        let aging_15 = s.population.cohort_15_44 / COHORT_15_44_DURATION;
        // aging_45_to_65 = cohort_45_64 / 20
        let aging_45 = s.population.cohort_45_64 / COHORT_45_64_DURATION;
        // These aging flows should all be positive
        assert!(aging_0 > 0.0);
        assert!(aging_15 > 0.0);
        assert!(aging_45 > 0.0);
        // Verify cohort 15-44 balance: d = aging_in - aging_out - deaths
        // deaths_15_44 = cohort_15_44 × M2(LE)
        let m2 = tables.mortality_15_44.eval(s.population.life_expectancy);
        let deaths_15_44 = s.population.cohort_15_44 * m2;
        let expected = aging_0 - aging_15 - deaths_15_44;
        assert_relative_eq!(d.d_cohort_15_44, expected, max_relative = 1e-10);
    }

    #[test]
    fn test_births_proportional_to_fertile_women() {
        let (mut s1, params, tables) = setup();
        population_derivatives(&mut s1, &params, &tables);
        let births1 = s1.population.birth_rate * s1.population.population;

        let (mut s2, _, _) = setup();
        let original_cohort = s2.population.cohort_15_44;
        s2.population.cohort_15_44 = original_cohort * 2.0;
        s2.population.population += original_cohort; // total grows by original cohort size
        population_derivatives(&mut s2, &params, &tables);
        let births2 = s2.population.birth_rate * s2.population.population;

        // Doubling fertile-age women should roughly double births
        // (not exactly due to crowding/food ratio changes from higher pop)
        assert!(births2 > births1 * 1.5,
            "births2 ({births2}) should be > 1.5 × births1 ({births1})");
    }

    #[test]
    fn test_all_derivatives_finite() {
        let (mut s, params, tables) = setup();
        let d = population_derivatives(&mut s, &params, &tables);
        assert!(d.d_cohort_0_14.is_finite());
        assert!(d.d_cohort_15_44.is_finite());
        assert!(d.d_cohort_45_64.is_finite());
        assert!(d.d_cohort_65_plus.is_finite());
        assert!(d.d_perceived_le.is_finite());
        assert!(d.d_ehspc.is_finite());
    }

    #[test]
    fn test_perceived_le_delay3_stages() {
        let (mut s, params, tables) = setup();
        // Set stages in a cascade: stage1 partly caught up, stage2 lagging, output lagging most.
        s.population.perceived_le_stage1 = 30.0;
        s.population.perceived_le_stage2 = 25.0;
        s.population.perceived_le = 20.0;
        let d = population_derivatives(&mut s, &params, &tables);
        // All stages should be moving upward (each stage's input > its current value)
        assert!(d.d_perceived_le_stage1 > 0.0, "stage1 should increase toward LE");
        assert!(d.d_perceived_le_stage2 > 0.0, "stage2 should increase toward stage1");
        assert!(d.d_perceived_le > 0.0, "perceived_le should increase toward stage2");
        // Verify Delay3 structure: each stage driven by (input - self) / tau
        let tau = 20.0 / 3.0;
        let le = s.population.life_expectancy;
        assert_relative_eq!(d.d_perceived_le_stage1, (le - 30.0) / tau, max_relative = 1e-10);
        assert_relative_eq!(d.d_perceived_le_stage2, (30.0 - 25.0) / tau, max_relative = 1e-10);
        assert_relative_eq!(d.d_perceived_le, (25.0 - 20.0) / tau, max_relative = 1e-10);
    }

    #[test]
    fn test_compensatory_fertility_at_low_perceived_le() {
        let (mut s_low, params, tables) = setup();
        // Low perceived LE → CMPLE > 1 → higher fertility
        s_low.population.perceived_le = 20.0;
        population_derivatives(&mut s_low, &params, &tables);
        let fert_low_ple = s_low.population.fertility_rate;

        let (mut s_high, _, _) = setup();
        // High perceived LE → CMPLE ≈ 1 → baseline fertility
        s_high.population.perceived_le = 60.0;
        population_derivatives(&mut s_high, &params, &tables);
        let fert_high_ple = s_high.population.fertility_rate;

        assert!(fert_low_ple > fert_high_ple,
            "fertility at low perceived LE ({fert_low_ple}) should exceed high perceived LE ({fert_high_ple})");
    }

    #[test]
    fn test_dcfs_calibrated_values() {
        let tables = WorldLookupTables::load();
        // Calibrated DCFS: shaped for Delay3 perceived-LE + historical fit.
        // Low at DIOPC=0 (less early growth), peaks at 200 (mid-income boom),
        // declines at high DIOPC (demographic transition).
        let dcfs_0 = tables.desired_family_size.eval(0.0);
        assert!((dcfs_0 - 2.85).abs() < 0.01,
            "DCFS(0) = {} should be ~2.85", dcfs_0);
        let dcfs_200 = tables.desired_family_size.eval(200.0);
        assert!((dcfs_200 - 3.50).abs() < 0.01,
            "DCFS(200) = {} should be ~3.50", dcfs_200);
        let dcfs_800 = tables.desired_family_size.eval(800.0);
        assert!((dcfs_800 - 1.90).abs() < 0.01,
            "DCFS(800) = {} should be ~1.90", dcfs_800);
    }
}
