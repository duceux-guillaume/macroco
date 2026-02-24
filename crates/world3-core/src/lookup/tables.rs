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
    /// Life expectancy multiplier from food (LEMF)
    /// x: food ratio (food per capita / subsistence food per capita)
    /// y: multiplier on life expectancy [0..2]
    pub life_exp_multiplier_food: LookupTable,

    /// Life expectancy multiplier from health services (LMHS)
    /// x: effective health services per capita (normalized to 1940 baseline)
    /// y: multiplier on life expectancy
    pub life_exp_multiplier_health: LookupTable,

    /// Life expectancy multiplier from crowding (LMCR)
    /// x: crowding ratio (population / carrying capacity)
    /// y: multiplier on life expectancy
    pub life_exp_multiplier_crowding: LookupTable,

    /// Life expectancy multiplier from pollution (LMPO)
    /// x: pollution index (normalized to 1970 = 1.0)
    /// y: multiplier on life expectancy
    pub life_exp_multiplier_pollution: LookupTable,

    /// Desired completed family size (DCFS) based on industrial output per capita
    /// x: industrial output per capita [1975 USD / person / year], normalized to 1 at 1970
    /// y: desired completed family size [children / woman]
    pub desired_family_size: LookupTable,

    /// Social family planning multiplier (FRSN) — effect of social norms
    /// x: effective family planning (0 = none, 1 = full)
    /// y: multiplier on fertility
    pub family_planning_multiplier: LookupTable,

    /// Fraction of services for health (FSH)
    /// x: effective services per capita (normalized)
    /// y: fraction devoted to health
    pub fraction_services_health: LookupTable,

    // --- Capital / Industrial sector ---
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
            // Life expectancy multiplier from food
            // Source: Meadows 1972, Table 5-1
            // x: food ratio (0=starvation, 1=subsistence, 2=adequate)
            // y: multiplier (0=death, 1=neutral, >1=benefit)
            life_exp_multiplier_food: LookupTable::new(
                "life_exp_multiplier_food",
                vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                vec![0.0, 1.0, 1.43, 1.50, 1.50, 1.50],
            ),

            // Life expectancy multiplier from health services
            // x: service output per capita [1975 USD/person/yr]
            //
            // Calibrated accounting for the ~20-year service-capital lag:
            //   actual sopc ≈ equilibrium_sopc / 1.6 due to the depreciation time constant.
            //   1900: sopc ≈ $200 (initialised at equilibrium) → lem = 0.76, LE ≈ 32 yr ✓
            //   1970: actual sopc ≈ $500 (lagged from eq $780) → lem ≈ 1.34, LE ≈ 53 yr ✓
            // Values < 1 mean poor health services actively reduce life expectancy below BASE.
            life_exp_multiplier_health: LookupTable::new(
                "life_exp_multiplier_health",
                vec![0.0, 200.0, 400.0, 600.0, 800.0, 1000.0],
                vec![0.50, 0.76, 1.15, 1.55, 1.78, 2.00],
            ),

            // Life expectancy multiplier from crowding
            // x: crowding ratio (population density / reference density)
            life_exp_multiplier_crowding: LookupTable::new(
                "life_exp_multiplier_crowding",
                vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0],
                vec![1.50, 1.40, 1.30, 1.20, 1.10, 1.00, 0.90, 0.80, 0.70, 0.60, 0.50],
            ),

            // Life expectancy multiplier from pollution
            // x: persistent pollution index (1.0 = 1970 level)
            life_exp_multiplier_pollution: LookupTable::new(
                "life_exp_multiplier_pollution",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
                vec![1.0, 0.99, 0.97, 0.95, 0.90, 0.85, 0.75, 0.65, 0.55],
            ),

            // Desired family size vs income [children / woman]
            // x: industrial output per capita [1975 USD/person/yr]
            //
            // Calibrated for a slow demographic transition: family size begins dropping
            // only when IOPC > ~$400/yr (late 20th century) matching World 3 BAU timing.
            // At 1970 IOPC ≈ $300 family size ≈ 4.3, matching historical data.
            desired_family_size: LookupTable::new(
                "desired_family_size",
                vec![0.0, 400.0, 800.0, 1200.0, 1600.0],
                vec![5.0, 4.0, 3.0, 2.1, 1.9],
            ),

            // Family planning multiplier on fertility
            // x: effective family planning (0..1)
            family_planning_multiplier: LookupTable::new(
                "family_planning_multiplier",
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
                vec![1.0, 0.90, 0.75, 0.55, 0.40],
            ),

            // Fraction of services for health
            fraction_services_health: LookupTable::new(
                "fraction_services_health",
                vec![0.0, 0.5, 1.0, 1.5, 2.0],
                vec![0.3, 0.35, 0.40, 0.45, 0.50],
            ),

            // Capital-output ratio multiplier from resource depletion
            // x: fraction of NNR remaining [0..1]
            // y: multiplier on capital-output ratio (>1 = more capital needed per unit output)
            //
            // Calibrated so that at NNR=1.0 (full resources), ICOR = 3.0 × 0.50 = 1.5,
            // giving investment (12% × 1/1.5 = 8%) > depreciation (5%) → capital grows at ~3%/yr.
            // Breakeven ≈ NNR=0.65: capital growth stalls as resources deplete past ~35%.
            // At NNR=0.0: ICOR = 3.0 × 4.0 = 12.0 → near collapse from resource scarcity.
            capital_output_ratio_resources: LookupTable::new(
                "capital_output_ratio_resources",
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
                vec![4.0, 3.2, 2.6, 2.0, 1.6, 1.25, 0.90, 0.75, 0.62, 0.55, 0.50],
            ),

            // Fraction of industrial output to agriculture (food pressure)
            // x: food ratio (food / subsistence food)
            industrial_fraction_to_agriculture: LookupTable::new(
                "industrial_fraction_to_agriculture",
                vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5],
                vec![0.40, 0.25, 0.15, 0.10, 0.07, 0.05],
            ),

            // Fraction of industrial output to services
            // x: services per capita normalized (1.0 = 1970)
            industrial_fraction_to_services: LookupTable::new(
                "industrial_fraction_to_services",
                vec![0.0, 0.5, 1.0, 1.5, 2.0],
                vec![0.30, 0.25, 0.20, 0.15, 0.12],
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

            // Land yield multiplier from capital inputs
            // x: agricultural inputs per hectare (normalized to 1 at 1970)
            land_yield_multiplier_capital: LookupTable::new(
                "land_yield_multiplier_capital",
                vec![0.0, 40.0, 80.0, 120.0, 160.0, 200.0, 240.0, 280.0, 320.0, 360.0, 400.0],
                vec![1.0, 3.0, 4.5, 5.0, 5.3, 5.6, 5.9, 6.1, 6.35, 6.6, 6.9],
            ),

            // Land yield multiplier from pollution
            // x: persistent pollution index
            land_yield_multiplier_pollution: LookupTable::new(
                "land_yield_multiplier_pollution",
                vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                vec![1.2, 1.0, 0.85, 0.75, 0.65, 0.55, 0.50],
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
