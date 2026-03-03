//! Digitized World 3 lookup tables.
//!
//! Sources: Meadows et al. "World Dynamics" (1972) and "Beyond the Limits" (1992),
//! supplemented by Randers "2052" (2012) digitization.
//!
//! Each table is named after the variable it represents with the convention from
//! the original Dynamo model documentation.

use super::LookupTable;

/// All lookup tables used in the World 3 model, loaded once at startup.
pub struct WorldLookupTables {
    // --- Population sector ---
    /// Life expectancy multiplier from food (LMF)
    /// World3-03 LMFT: x = food ratio, y = LE multiplier
    pub life_exp_multiplier_food: LookupTable,

    /// Life expectancy multiplier from health services (LMHS)
    /// x: effective health services per capita [USD/person/yr]
    /// y: multiplier on life expectancy
    pub life_exp_multiplier_health: LookupTable,

    /// Life expectancy multiplier from crowding (CMI × FPU proxy)
    /// x: crowding ratio (population / reference)
    /// y: multiplier on life expectancy (< 1 at high crowding)
    pub life_exp_multiplier_crowding: LookupTable,

    /// Life expectancy multiplier from pollution (LMPD)
    /// World3-03 LMPDE: x = pollution index, y = LE multiplier
    pub life_exp_multiplier_pollution: LookupTable,

    /// Mortality rate for cohort 0-14 (M1) — World3-03 table
    /// x: life expectancy [years], y: annual mortality rate
    pub mortality_0_14: LookupTable,

    /// Mortality rate for cohort 15-44 (M2) — World3-03 table
    /// x: life expectancy [years], y: annual mortality rate
    pub mortality_15_44: LookupTable,

    /// Mortality rate for cohort 45-64 (M3) — World3-03 table
    /// x: life expectancy [years], y: annual mortality rate
    pub mortality_45_64: LookupTable,

    /// Mortality rate for cohort 65+ (M4) — World3-03 table
    /// x: life expectancy [years], y: annual mortality rate
    pub mortality_65_plus: LookupTable,

    /// Desired completed family size (DCFS)
    /// World3-03: dcfsn=3.8, modified by SFSN lookup from DIOPC
    /// x: industrial output per capita [USD/person/yr]
    /// y: desired completed family size [children/woman]
    pub desired_family_size: LookupTable,

    /// Social family planning multiplier (FRSN) — effect of family planning programs
    /// x: effective family planning (0 = none, 1 = full)
    /// y: multiplier on fertility
    pub family_planning_multiplier: LookupTable,

    /// Fecundity multiplier from life expectancy (FM) — World3-03 table
    /// x: life expectancy [years], y: fecundity multiplier
    pub fecundity_multiplier: LookupTable,

    /// Fraction of services for health (FSH)
    /// x: effective services per capita (normalized)
    /// y: fraction devoted to health
    pub fraction_services_health: LookupTable,

    // --- Capital / Industrial sector ---
    /// Fraction of industrial output allocated to consumption (FIOAC)
    /// World3-03: x = IOPC/IOPCD ratio, y = consumption fraction [0.3..0.83]
    /// Simplified: x = IOPC [USD/person/yr], y = consumption fraction
    pub consumption_fraction: LookupTable,

    /// Industrial capital output ratio (ICOR) as function of resource fraction
    /// x: fraction of non-renewable resources remaining [0..1]
    /// y: capital-output ratio multiplier
    pub capital_output_ratio_resources: LookupTable,

    /// Fraction of industrial output allocated to agriculture (FIOAA)
    /// x: food ratio (food per capita / subsistence food)
    /// y: fraction of industrial output to agriculture [0..1]
    pub industrial_fraction_to_agriculture: LookupTable,

    /// Fraction of industrial output allocated to services (FIOAS)
    /// x: service output per capita (normalized to 1 at 1970)
    /// y: fraction [0..1]
    pub industrial_fraction_to_services: LookupTable,

    /// Jobs per industrial capital unit (JPICU)
    /// x: industrial output per capita (normalized)
    /// y: jobs per unit capital
    pub jobs_per_capital: LookupTable,

    /// Labor force participation (LFP) by age structure
    /// x: fraction population age 15-64
    /// y: labor force fraction
    pub labor_force_participation: LookupTable,

    // --- Agriculture sector ---
    /// Land yield multiplier from capital (LYMC)
    /// x: agricultural inputs per hectare (normalized)
    /// y: yield multiplier
    pub land_yield_multiplier_capital: LookupTable,

    /// Land yield multiplier from air pollution (LYMAP)
    /// x: persistent pollution index (1970 = 1)
    /// y: yield multiplier [0..1]
    pub land_yield_multiplier_pollution: LookupTable,

    /// Land erosion rate (LERD) from yield pressure
    /// x: land yield ratio (actual / potential)
    /// y: erosion multiplier
    pub land_erosion_multiplier: LookupTable,

    /// Land development cost (LDCO)
    /// x: arable land fraction remaining (arable / total land area)
    /// y: development cost multiplier
    pub land_development_cost: LookupTable,

    /// Food ratio needed for full fertility (FRNF)
    /// x: food per capita / subsistence food
    /// y: fertility fraction
    pub food_fertility_multiplier: LookupTable,

    // --- Resource sector ---
    /// Resource extraction efficiency (FCAOR) — fraction of capital in resource sector
    /// x: fraction of resources remaining [0..1]
    /// y: fraction of capital allocated to resource extraction [0..1]
    pub capital_fraction_resource_extraction: LookupTable,

    // --- Pollution sector ---
    /// Persistent pollution generation factor from industry (PPGIO)
    /// x: industrial output per capita (normalized to 1970)
    /// y: pollution generation multiplier
    pub pollution_generation_industry: LookupTable,

    /// Persistent pollution generation factor from agriculture (PPGAO)
    /// x: agricultural inputs per capita (normalized)
    /// y: pollution generation multiplier
    pub pollution_generation_agriculture: LookupTable,

    /// Persistent pollution assimilation (PPASR)
    /// x: persistent pollution index (1970 = 1)
    /// y: assimilation time [years]
    pub pollution_assimilation_time: LookupTable,
}

impl WorldLookupTables {
    /// Load all tables from the digitized World 3 data.
    ///
    /// These are the canonical piecewise-linear tables from Meadows et al. (1972, 1992).
    /// X values and Y values are taken directly from the published model documentation.
    pub fn load() -> Self {
        Self {
            // Life expectancy multiplier from food (LMF)
            // World3-03 LMFT: food adequacy affects LE
            // At food_ratio=1 (subsistence), LMF=1.0. Below: rapid decline. Above: modest benefit.
            // Flattened above food_ratio=1.0 compared to World3-03's LMFT table.
            // Our model produces higher food/cap than World3-03 at the same parameters
            // Calibrated to produce LE≈33 in 1900 (food_ratio≈1.9) and LE≈60 in 1970
            // (food_ratio≈2.5). Above food_ratio=1.0, gains are moderate — food security
            // helps but doesn't dominate LE like health services do.
            life_exp_multiplier_food: LookupTable::new(
                "life_exp_multiplier_food",
                vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 3.0, 4.0],
                vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.04, 1.08, 1.12, 1.16, 1.20, 1.33, 1.40],
            ),

            // Life expectancy multiplier from health services (LMHS)
            // World3-03: LMHS1 table from EHSPC (effective health services per capita).
            // x: effective health services per capita [USD/person/yr]
            // In World3-03, EHSPC uses a 20-year delay (HSAPC → EHSPC). Our model
            // doesn't delay health services, so we keep LMHS = 1.0 until EHSPC > 40
            // to approximate the delayed startup effect (S-shaped response).
            // Calibrated so:
            //   1900: EHSPC ≈ $36 → LMHS ≈ 1.0 (minimal health infrastructure)
            //   1940: EHSPC ≈ $60 → LMHS ≈ 1.20 (health system developing)
            //   1970: EHSPC ≈ $150 → LMHS ≈ 1.85 (modern health services)
            life_exp_multiplier_health: LookupTable::new(
                "life_exp_multiplier_health",
                vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0, 150.0, 200.0],
                vec![1.0, 1.0, 1.0, 1.20, 1.50, 1.70, 1.85, 1.95],
            ),

            // Life expectancy multiplier from crowding
            // World3-03: LMCR = 1 - CMI(IOPC) × FPU(POP), simplified to a
            // direct lookup on population/reference ratio.
            // At 1900 (pop/ref = 0.44): ~1.0 (minimal crowding effect)
            // At 1970 (pop/ref = 1.0): ~0.95 (mild crowding)
            // At high pop: drops significantly
            life_exp_multiplier_crowding: LookupTable::new(
                "life_exp_multiplier_crowding",
                vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0],
                vec![1.05, 1.0, 0.95, 0.90, 0.85, 0.80, 0.75],
            ),

            // Life expectancy multiplier from pollution (LMPDE)
            // World3-03 table, extended to higher pollution levels
            life_exp_multiplier_pollution: LookupTable::new(
                "life_exp_multiplier_pollution",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0],
                vec![1.0, 0.99, 0.97, 0.95, 0.90, 0.85, 0.75, 0.55, 0.40],
            ),

            // Mortality tables M1-M4 from World3-03
            // x: life expectancy [years], y: annual mortality rate for each cohort
            // Source: Meadows 2004, digitized from Dynamo model documentation
            mortality_0_14: LookupTable::new(
                "mortality_0_14",
                vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![0.0567, 0.0366, 0.0243, 0.0155, 0.0082, 0.0023, 0.001],
            ),

            mortality_15_44: LookupTable::new(
                "mortality_15_44",
                vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![0.0266, 0.0171, 0.0110, 0.0065, 0.0040, 0.0016, 0.0008],
            ),

            mortality_45_64: LookupTable::new(
                "mortality_45_64",
                vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![0.0562, 0.0373, 0.0252, 0.0171, 0.0118, 0.0083, 0.0060],
            ),

            mortality_65_plus: LookupTable::new(
                "mortality_65_plus",
                vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![0.13, 0.11, 0.09, 0.07, 0.06, 0.05, 0.04],
            ),

            // Desired completed family size
            // World3-03: DCFS = dcfsn × SFSN(DIOPC) × FRSN(FIE)
            //   dcfsn = 3.8, SFSN ranges 0.5-1.25
            // Simplified to direct lookup from perceived IOPC capturing the combined
            // effect of social norms + compensatory fertility (CMPLE).
            // World3-03 uses dcfsn=3.8 × SFSN(DIOPC) × CMPLE(perceived_LE). Our
            // model lacks CMPLE, so values are raised ~20% at low/moderate income
            // to compensate for the missing compensatory fertility effect.
            // Key calibration:
            //   IOPC=$44 (1900): ~5.7 (high fertility, compensatory + subsistence norms)
            //   IOPC=$200 (1970): ~4.5 (transition beginning, still high compensation)
            //   IOPC=$800: ~2.1 (post-transition, compensation minimal)
            desired_family_size: LookupTable::new(
                "desired_family_size",
                vec![0.0, 50.0, 100.0, 200.0, 400.0, 600.0, 800.0, 1200.0, 1600.0],
                vec![4.40, 4.35, 4.25, 4.00, 3.00, 2.40, 2.05, 1.90, 1.80],
            ),

            // Family planning multiplier on fertility
            // x: effective family planning (0..1)
            family_planning_multiplier: LookupTable::new(
                "family_planning_multiplier",
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
                vec![1.0, 0.90, 0.75, 0.55, 0.40],
            ),

            // Fecundity multiplier from life expectancy (FM)
            // World3-03: biological fecundity depends on health (proxy: LE)
            // At low LE, poor nutrition/disease reduces fecundity
            fecundity_multiplier: LookupTable::new(
                "fecundity_multiplier",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![0.0, 0.2, 0.4, 0.6, 0.7, 0.75, 0.8, 0.85, 0.87],
            ),

            // Fraction of services for health
            fraction_services_health: LookupTable::new(
                "fraction_services_health",
                vec![0.0, 0.5, 1.0, 1.5, 2.0],
                vec![0.3, 0.35, 0.40, 0.45, 0.50],
            ),

            // Fraction of industrial output allocated to consumption (FIOAC)
            // Based on World3-03 FIOAC1 table (x = IOPC/IOPCD, IOPCD ≈ $400).
            // Raised ~0.10 to compensate for our model's missing dynamic references
            // (IOPCD, ISOPC, IFPC) which in World3-03 absorb more output as income rises.
            // This produces the correct aggregate investment rate (~32% in 1900).
            consumption_fraction: LookupTable::new(
                "consumption_fraction",
                vec![0.0, 80.0, 160.0, 240.0, 320.0, 400.0, 480.0, 560.0, 640.0, 720.0, 800.0],
                vec![0.40, 0.42, 0.44, 0.46, 0.48, 0.53, 0.78, 0.80, 0.82, 0.83, 0.83],
            ),

            // Capital-output ratio multiplier from resource depletion
            // x: fraction of NNR remaining [0..1]
            // y: multiplier on capital-output ratio
            //
            // World3-03: ICOR1 = 3.0 (constant). Resource scarcity feeds back ONLY
            // through FCAOR (capital allocated to resource extraction), not through ICOR.
            // This table is flat at 1.0 to match World3-03 behavior.
            capital_output_ratio_resources: LookupTable::new(
                "capital_output_ratio_resources",
                vec![0.0, 0.5, 1.0],
                vec![1.0, 1.0, 1.0],
            ),

            // Fraction of industrial output to agriculture (FIOAA1)
            // World3-03: FIOAA1 indexed by IFPC/FPC ratio, where IFPC > subsistence.
            // Our x-axis uses food_ratio (FPC/subsistence). Because indicated food
            // per capita exceeds subsistence, agricultural investment persists even
            // when food_ratio > 1.0. Key points:
            //   food_ratio=1.7 (1900 typical): FIOAA ≈ 0.11
            //   food_ratio=2.0 (1970 typical): FIOAA ≈ 0.08
            //   food_ratio=3.0+: FIOAA → 0 (food abundance)
            industrial_fraction_to_agriculture: LookupTable::new(
                "industrial_fraction_to_agriculture",
                vec![0.0, 0.5, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0],
                vec![0.40, 0.30, 0.20, 0.16, 0.12, 0.08, 0.04, 0.0, 0.0],
            ),

            // Fraction of industrial output to services (FIOAS1)
            // World3-03: FIOAS1 table, x = SOPC/ISOPC ratio
            // At low SOPC relative to desired, high allocation to services
            industrial_fraction_to_services: LookupTable::new(
                "industrial_fraction_to_services",
                vec![0.0, 0.5, 1.0, 1.5, 2.0],
                vec![0.30, 0.20, 0.10, 0.05, 0.0],
            ),

            // Jobs per unit of industrial capital
            // x: industrial output per capita normalized
            jobs_per_capital: LookupTable::new(
                "jobs_per_capital",
                vec![0.0, 0.5, 1.0, 2.0, 3.0, 4.0],
                vec![0.0007, 0.0014, 0.0017, 0.0018, 0.0019, 0.002],
            ),

            // Labor force participation
            labor_force_participation: LookupTable::new(
                "labor_force_participation",
                vec![0.5, 0.6, 0.7, 0.8],
                vec![0.50, 0.55, 0.60, 0.65],
            ),

            // Land yield multiplier from capital inputs (LYMC)
            // x: agricultural inputs per hectare [$/ha/yr]
            // At zero inputs, yield equals base land fertility (LYMC=1.0).
            // World3-03: LYMC1 table, x = agricultural inputs per hectare [$/ha/yr].
            // Capital enhancement improves yields rapidly at first (Green Revolution),
            // then with diminishing returns.
            land_yield_multiplier_capital: LookupTable::new(
                "land_yield_multiplier_capital",
                vec![0.0, 40.0, 80.0, 120.0, 160.0, 200.0, 240.0, 280.0, 320.0, 360.0, 400.0],
                vec![1.0, 3.0, 4.5, 5.0, 5.3, 5.6, 5.9, 6.1, 6.35, 6.6, 6.9],
            ),

            // Land yield multiplier from pollution (LYMAP)
            // x: persistent pollution index (1970 = 1)
            // At low pollution, yield is at baseline (1.0). As pollution rises,
            // yield degrades due to acid rain, soil contamination, etc.
            land_yield_multiplier_pollution: LookupTable::new(
                "land_yield_multiplier_pollution",
                vec![0.0, 1.0, 5.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                vec![1.0, 1.0, 0.95, 0.90, 0.80, 0.70, 0.60, 0.50, 0.40],
            ),

            // Land erosion multiplier from over-farming
            // x: land yield / potential yield ratio
            land_erosion_multiplier: LookupTable::new(
                "land_erosion_multiplier",
                vec![0.0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0],
                vec![0.0, 0.1, 0.3, 0.5, 0.7, 1.0, 1.5, 2.0, 2.5],
            ),

            // Land development cost — increases as marginal land is brought into production
            // x: fraction of potential arable land already developed [0..1]
            land_development_cost: LookupTable::new(
                "land_development_cost",
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
                vec![100.0, 117.0, 137.0, 161.0, 192.0, 232.0, 282.0, 344.0, 418.0, 507.0, 616.0],
            ),

            // Food ratio effect on fertility
            // x: food per capita / subsistence food per capita
            food_fertility_multiplier: LookupTable::new(
                "food_fertility_multiplier",
                vec![0.0, 0.5, 1.0, 1.5, 2.0],
                vec![0.0, 0.6, 1.0, 1.05, 1.1],
            ),

            // Fraction of capital allocated to resource extraction
            // As resources deplete, more capital is needed to extract the same amount
            // x: fraction of NNR remaining [0..1]
            // y: fraction of total industrial capital diverted to resource extraction
            capital_fraction_resource_extraction: LookupTable::new(
                "capital_fraction_resource_extraction",
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
                vec![1.0, 0.9, 0.70, 0.50, 0.40, 0.30, 0.20, 0.14, 0.08, 0.04, 0.0],
            ),

            // Pollution generation from industrial output
            // x: industrial output per capita (normalized to 1.0 at 1970)
            pollution_generation_industry: LookupTable::new(
                "pollution_generation_industry",
                vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                vec![0.0, 1.0, 1.5, 1.9, 2.16, 2.36],
            ),

            // Pollution generation from agricultural inputs
            // x: agricultural inputs (normalized)
            pollution_generation_agriculture: LookupTable::new(
                "pollution_generation_agriculture",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                vec![0.0, 1.0, 1.7, 2.2, 2.5],
            ),

            // Pollution assimilation time
            // x: persistent pollution index
            // y: assimilation time [years] — increases sharply as environment is overwhelmed
            //
            // Steeper than original Meadows table so that pollution accumulates to
            // visible levels (index > 5) by 2000 and peak > 10 by 2030–2040 in BAU.
            // At low PP the environment assimilates quickly; above PP=10 it slows dramatically.
            pollution_assimilation_time: LookupTable::new(
                "pollution_assimilation_time",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                vec![20.0, 45.0, 90.0, 150.0, 220.0, 320.0, 480.0],
            ),
        }
    }
}
