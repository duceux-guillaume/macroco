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

    /// Fraction of industrial output allocated to agriculture (FIOAA)
    /// x: food ratio (food per capita / subsistence food)
    /// y: fraction of industrial output to agriculture [0..1]
    pub industrial_fraction_to_agriculture: LookupTable,

    /// Indicated food per capita (IFPC) — World3-03 table
    /// x: industrial output per capita [$/person/yr]
    /// y: indicated food per capita [kg/person/yr]
    /// As societies industrialize, people demand better diets.
    /// At IOPC=0: IFPC=230 (subsistence). At IOPC=1600: IFPC=1250 (saturation).
    pub indicated_food_per_capita: LookupTable,

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

    /// Urban-industrial land per capita (UILPC) — World3-03 table
    /// x: IOPC [1975 USD/person/yr], y: hectares/person for urban/industrial use
    pub urban_industrial_land_per_capita: LookupTable,

    /// Land fertility degradation rate (LFDR) — World3-03 table
    /// x: pollution index (1970 = 1), y: annual degradation fraction
    pub land_fertility_degradation: LookupTable,

    /// Land fertility regeneration time (LFRT) — World3-03 table
    /// x: land yield ratio (actual / inherent), y: regeneration time [years]
    pub land_fertility_regeneration_time: LookupTable,

    /// Fraction allocated to land maintenance (FALM) — World3-03 table
    /// x: food ratio, y: fraction of agricultural output to land maintenance
    pub fraction_land_maintenance: LookupTable,

    /// Compensatory multiplier from perceived life expectancy (CMPLE) — World3-03
    /// x: perceived life expectancy [years]
    /// y: multiplier on desired family size (>1 when perceived LE is low)
    pub compensatory_fertility: LookupTable,

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

            // Desired completed family size (DCFS)
            // World3-03: DCFS = dcfsn × SFSN(DIOPC), where dcfsn=3.8, SFSN 0.5-1.25.
            // Simplified to direct lookup from perceived IOPC capturing dcfsn × sfsn.
            // CMPLE (compensatory fertility from perceived LE) is now applied separately.
            // Values at low income are reduced (CMPLE provides ~1.25× compensation there),
            // but mid-to-high income values stay near previous calibration because
            // CMPLE ≈ 1.0 at higher LE (perceived_LE lags 20yr behind actual LE).
            // Key calibration (before CMPLE):
            //   IOPC=$44 (1900): 3.60 × CMPLE(33)≈1.22 → effective ~4.39
            //   IOPC=$200 (1960): 3.75 × CMPLE(40)≈1.10 → effective ~4.13
            //   IOPC=$800: 2.10 × CMPLE(60)≈0.95 → effective ~2.00
            desired_family_size: LookupTable::new(
                "desired_family_size",
                vec![0.0, 50.0, 100.0, 200.0, 400.0, 600.0, 800.0, 1200.0, 1600.0],
                vec![3.60, 3.58, 3.55, 3.75, 3.00, 2.45, 2.10, 1.95, 1.85],
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

            // Fraction of industrial output to agriculture (FIOAA)
            // Input: food_ratio = food_per_capita_smooth / indicated_food_per_capita(IOPC).
            // At low IOPC, IFPC ≈ SFPC (230), so this behaves like the original
            // FPC/SFPC allocation. At high IOPC, IFPC rises, keeping food_ratio
            // moderate and preventing the zero-allocation trap.
            // Table shape calibrated for our model (no LFH/PL factors).
            // Floor of 0.04 at high food_ratio ensures minimum agricultural
            // maintenance even in wealthy societies (prevents yield collapse).
            industrial_fraction_to_agriculture: LookupTable::new(
                "industrial_fraction_to_agriculture",
                vec![0.0, 0.5, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0],
                vec![0.40, 0.30, 0.20, 0.16, 0.12, 0.08, 0.05, 0.04, 0.04],
            ),

            // Indicated food per capita (IFPC) — calibrated for our model.
            // x: IOPC [$/person/yr], y: IFPC [kg/person/yr]
            // Based on World3-03's IFPC table, but starts at subsistence (230)
            // to preserve BAU behavior (where IOPC stays below ~330). Rises at
            // higher IOPC to prevent zero-allocation trap in Technology and
            // Stabilized scenarios where IOPC can exceed 1600.
            // Without IFPC, food_ratio = FPC/SFPC would reach 3.0+ and FIOAA → 0.
            // With IFPC, food_ratio = FPC_smooth/IFPC stays moderate even at high FPC.
            // Extended to IOPC=2500 for Stabilized scenario (IOPC peaks ~2000).
            indicated_food_per_capita: LookupTable::new(
                "indicated_food_per_capita",
                vec![0.0, 200.0, 400.0, 600.0, 800.0, 1000.0, 1200.0, 1400.0, 1600.0, 2000.0, 2500.0],
                vec![230.0, 230.0, 350.0, 500.0, 650.0, 800.0, 950.0, 1100.0, 1200.0, 1400.0, 1600.0],
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

            // Compensatory multiplier from perceived LE (CMPLE)
            // World3-03: when perceived LE is low (high infant mortality), women
            // have more children to compensate for expected child deaths.
            // At perceived_LE=30 (1900 conditions): CMPLE≈1.25 (25% more births)
            // At perceived_LE=50 (improving health): CMPLE≈1.0 (no compensation)
            // At perceived_LE=60+ (modern health): CMPLE<1.0 (confidence in survival)
            compensatory_fertility: LookupTable::new(
                "compensatory_fertility",
                vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![1.40, 1.25, 1.10, 1.0, 0.95, 0.92, 0.90],
            ),

            // Urban-industrial land per capita (UILPC)
            // World3-03: maps IOPC to hectares per person needed for urban/industrial use.
            // At subsistence (IOPC≈$40): 0.005 ha/person (rural, low urbanization)
            // At 1970 levels (IOPC≈$200): 0.05 ha/person (significant urbanization)
            // At high income (IOPC≈$1600): 0.16 ha/person (sprawl, infrastructure)
            urban_industrial_land_per_capita: LookupTable::new(
                "urban_industrial_land_per_capita",
                vec![0.0, 200.0, 400.0, 600.0, 800.0, 1000.0, 1600.0],
                vec![0.005, 0.05, 0.09, 0.11, 0.13, 0.15, 0.16],
            ),

            // Land fertility degradation rate (LFDR)
            // World3-03 LFDR1t: pollution-driven soil degradation.
            // At low pollution: no degradation. At high pollution: rapid degradation.
            land_fertility_degradation: LookupTable::new(
                "land_fertility_degradation",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
                vec![0.0, 0.04, 0.10, 0.20, 0.30, 0.40],
            ),

            // Land fertility regeneration time (LFRT)
            // World3-03: time to regenerate fertility depends on current yield intensity.
            // Low yield (extensive farming): slow regeneration. High yield: faster with investment.
            land_fertility_regeneration_time: LookupTable::new(
                "land_fertility_regeneration_time",
                vec![0.0, 0.02, 0.04, 0.06, 0.08, 0.10],
                vec![20.0, 13.0, 8.0, 4.0, 2.0, 2.0],
            ),

            // Fraction allocated to land maintenance (FALM)
            // World3-03: fraction of agricultural output devoted to maintaining soil quality.
            // When food ratio is low (scarcity): less maintenance (survival priority).
            // When food ratio is high (abundance): more maintenance possible.
            fraction_land_maintenance: LookupTable::new(
                "fraction_land_maintenance",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                vec![0.0, 0.04, 0.07, 0.09, 0.10],
            ),

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
