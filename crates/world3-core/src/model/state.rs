//! World 3 complete state vector.
//!
//! `WorldState` is the `y` in the ODE system `dy/dt = f(t, y, params)`.
//! Every field has explicit units documented in the comment.
//!
//! The struct also implements `to_vec()` / `from_vec()` for use by the RK4
//! solver, which needs to perform scalar arithmetic on the state.

use serde::{Deserialize, Serialize};

/// Complete state of the World 3 model at a single point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Simulation year (e.g. 1900.0 … 2200.0)
    pub time: f64,

    pub population: PopulationState,
    pub capital: CapitalState,
    pub agriculture: AgricultureState,
    pub resources: ResourceState,
    pub pollution: PollutionState,
}

// ---------------------------------------------------------------------------
// Sector states
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopulationState {
    /// Total population [persons]
    pub population: f64,
    /// Age cohort 0–14 [persons]
    pub cohort_0_14: f64,
    /// Age cohort 15–44 [persons]
    pub cohort_15_44: f64,
    /// Age cohort 45–64 [persons]
    pub cohort_45_64: f64,
    /// Age cohort 65+ [persons]
    pub cohort_65_plus: f64,
    /// Crude birth rate [births / person / year]
    pub birth_rate: f64,
    /// Crude death rate [deaths / person / year]
    pub death_rate: f64,
    /// Life expectancy at birth [years]
    pub life_expectancy: f64,
    /// Total fertility rate [children / woman]
    pub fertility_rate: f64,
    /// Perceived life expectancy (20-year delay) [years]
    /// World3-03: PLE — drives compensatory fertility via CMPLE.
    pub perceived_le: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapitalState {
    /// Industrial capital stock [1975 USD]
    pub industrial_capital: f64,
    /// Service capital stock [1975 USD]
    pub service_capital: f64,
    /// Delayed IOPC for social norms (20-year lag) [1975 USD / person / year]
    /// World3-03: DIOPC — drives desired family size with social adjustment delay.
    pub perceived_iopc: f64,
    /// Industrial output [1975 USD / year]
    pub industrial_output: f64,
    /// Industrial output per capita [1975 USD / person / year]
    pub industrial_output_per_capita: f64,
    /// Service output per capita [1975 USD / person / year]
    pub service_output_per_capita: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgricultureState {
    /// Total arable land [hectares]
    pub arable_land: f64,
    /// Potentially arable but not yet developed [hectares]
    pub potentially_arable_land: f64,
    /// Urban-industrial land [hectares] — World3-03: uil, uili=8.2e6
    pub urban_industrial_land: f64,
    /// Land fertility [kg / hectare / year] — World3-03: lfert, lferti=600
    pub land_fertility: f64,
    /// Annual food production [vegetable-equivalent kg / year]
    pub food: f64,
    /// Food per capita [kg / person / year]
    pub food_per_capita: f64,
    /// Land yield [kg / hectare / year]
    pub land_yield: f64,
    /// Agricultural capital inputs [1975 USD / hectare / year]
    pub agricultural_inputs_per_hectare: f64,
    /// Smoothed food per capita [kg / person / year] — World3-03 FSPD=2yr delay.
    /// ODE stock: d(fpc_smooth)/dt = (fpc - fpc_smooth) / FSPD.
    /// Used by capital sector for allocation fractions (breaks oscillation).
    pub food_per_capita_smooth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceState {
    /// Non-renewable resources remaining [dimensionless, normalized to 1.0 in 1900]
    pub nonrenewable_resources: f64,
    /// Fraction of original resources remaining [0..1]
    pub fraction_remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PollutionState {
    /// Persistent pollution stock (appeared, after delay) [pollution units, 1970 = 1]
    pub persistent_pollution: f64,
    /// Pollution appearing pipeline (generated, in 20-year delay) [pollution units]
    pub pollution_appearance_buffer: f64,
    /// Pollution index (normalized to 1.0 in 1970)
    pub pollution_index: f64,
    /// Current pollution generation rate [units / year]
    pub generation_rate: f64,
    /// Current pollution assimilation rate [units / year]
    pub assimilation_rate: f64,
}

// ---------------------------------------------------------------------------
// Vec conversion for RK4 solver
// ---------------------------------------------------------------------------

impl WorldState {
    /// The number of state variables (excluding `time`, which is tracked separately).
    pub const N: usize = 16;

    /// Human Welfare Index (0–1 scale) — World3-03 composite indicator.
    ///
    /// Geometric mean of life expectancy index and income index, inspired by
    /// UNDP HDI methodology adapted for World3 variables. Education is proxied
    /// by income since World3 doesn't model it separately.
    ///
    /// - LEI = (LE - 25) / 60, clamped to [0, 1]
    /// - II  = (ln(IOPC) - ln(20)) / (ln(5000) - ln(20)), clamped to [0, 1]
    ///   (bounds in 1975 USD: $20 subsistence, $5000 high development)
    /// - HWI = sqrt(LEI × II)
    pub fn hwi(&self) -> f64 {
        let lei = ((self.population.life_expectancy - 25.0) / 60.0).clamp(0.0, 1.0);
        let iopc = self.capital.industrial_output_per_capita.max(1.0);
        let ii = ((iopc.ln() - 20.0_f64.ln()) / (5000.0_f64.ln() - 20.0_f64.ln()))
            .clamp(0.0, 1.0);
        (lei * ii).sqrt()
    }

    /// Ecological Footprint (1.0 = Earth's biocapacity) — World3-03 composite indicator.
    ///
    /// EF = (arable_land + UIL + absorption_land) / biocapacity_1970
    ///
    /// `absorption_land` represents the hypothetical land area needed to absorb
    /// current persistent pollution. Scaled so pollution_index=1.0 (1970 level)
    /// corresponds to ~0.3 billion hectares of absorption capacity.
    pub fn ecological_footprint(&self) -> f64 {
        const BIOCAPACITY_1970: f64 = 1.91e9; // hectares
        let land_use = self.agriculture.arable_land + self.agriculture.urban_industrial_land;
        let absorption_land = self.pollution.pollution_index * 0.3e9;
        (land_use + absorption_land) / BIOCAPACITY_1970
    }

    /// Extract the integrable state variables into a flat `Vec<f64>`.
    /// `time` is not included — the solver manages time separately.
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            // Population (4 cohorts — total is derived)
            self.population.cohort_0_14,
            self.population.cohort_15_44,
            self.population.cohort_45_64,
            self.population.cohort_65_plus,
            // Capital (3 stocks: IC, SC, perceived_iopc)
            self.capital.industrial_capital,
            self.capital.service_capital,
            self.capital.perceived_iopc,
            // Agriculture (5 stocks)
            self.agriculture.arable_land,
            self.agriculture.potentially_arable_land,
            self.agriculture.urban_industrial_land,
            self.agriculture.land_fertility,
            self.agriculture.food_per_capita_smooth,
            // Resources (1 stock)
            self.resources.nonrenewable_resources,
            // Pollution (2 stocks: appeared + pipeline)
            self.pollution.persistent_pollution,
            self.pollution.pollution_appearance_buffer,
            // Population delay (1 stock: perceived LE)
            self.population.perceived_le,
        ]
    }

    /// Reconstruct state from a flat vec (only the 15 ODE stocks).
    /// Derived/auxiliary fields are left at their defaults — they will be
    /// computed by the derivative function before use.
    pub fn from_vec(time: f64, v: &[f64]) -> Self {
        assert_eq!(v.len(), Self::N);
        let mut s = WorldState { time, ..Default::default() };

        s.population.cohort_0_14 = v[0].max(0.0);
        s.population.cohort_15_44 = v[1].max(0.0);
        s.population.cohort_45_64 = v[2].max(0.0);
        s.population.cohort_65_plus = v[3].max(0.0);
        s.population.population =
            s.population.cohort_0_14 + s.population.cohort_15_44
            + s.population.cohort_45_64 + s.population.cohort_65_plus;

        s.capital.industrial_capital = v[4].max(0.0);
        s.capital.service_capital = v[5].max(0.0);
        s.capital.perceived_iopc = v[6].max(0.0);

        s.agriculture.arable_land = v[7].max(0.0);
        s.agriculture.potentially_arable_land = v[8].max(0.0);
        s.agriculture.urban_industrial_land = v[9].max(0.0);
        s.agriculture.land_fertility = v[10].max(1.0); // never zero
        s.agriculture.food_per_capita_smooth = v[11].max(0.0);

        s.resources.nonrenewable_resources = v[12].max(0.0);
        s.resources.fraction_remaining = v[12].clamp(0.0, 1.0);

        s.pollution.persistent_pollution = v[13].max(0.0);
        s.pollution.pollution_appearance_buffer = v[14].max(0.0);

        s.population.perceived_le = v[15].max(5.0); // never below minimum LE
        s
    }

    /// World 3 initial conditions for year 1900.
    /// Values calibrated to broadly match Meadows 1972 standard run starting point.
    pub fn initial_1900() -> Self {
        WorldState {
            time: 1900.0,
            population: PopulationState {
                population: 1.6e9,
                // World3-03 initial cohorts (Meadows 2004): p1i=6.5e8, p2i=7.0e8, p3i=1.9e8, p4i=6.0e7
                cohort_0_14: 6.5e8,
                cohort_15_44: 7.0e8,
                cohort_45_64: 1.9e8,
                cohort_65_plus: 6.0e7,
                perceived_le: 33.0, // Initial perceived LE matches 1900 computed LE
                ..Default::default()
            },
            capital: CapitalState {
                industrial_capital: 2.1e11,  // World3-03: ici = 2.1e11 (1975 USD)
                service_capital: 1.44e11,    // World3-03: sci = 1.44e11 (1975 USD)
                // 1900 IOPC ≈ IC/ICOR/POP = 2.1e11 / 3.0 / 1.6e9 ≈ 43.75
                perceived_iopc: 43.75,
                ..Default::default()
            },
            agriculture: AgricultureState {
                arable_land: 0.9e9,            // hectares
                potentially_arable_land: 2.3e9,
                urban_industrial_land: 8.2e6,  // World3-03: uili = 8.2e6 hectares
                land_fertility: 600.0,         // World3-03: lferti = 600 kg/ha/yr
                food_per_capita: 400.0,        // initial estimate; overwritten by agriculture sector
                food_per_capita_smooth: 400.0, // matches initial fpc estimate
                ..Default::default()
            },
            resources: ResourceState {
                nonrenewable_resources: 1.0, // 100% remaining in 1900
                fraction_remaining: 1.0,
            },
            pollution: PollutionState {
                persistent_pollution: 0.05,
                // Steady-state buffer: at 1900, generation ≈ pp / assimilation_time
                // ≈ 0.05 / 20 = 0.0025/yr. Buffer = generation × delay = 0.0025 × 20 = 0.05.
                pollution_appearance_buffer: 0.05,
                pollution_index: 0.05,
                ..Default::default()
            },
        }
    }

    /// Return a zero state (for use as a derivative accumulator)
    pub fn zero_derivative(time: f64) -> Self {
        Self {
            time,
            population: PopulationState::default(),
            capital: CapitalState::default(),
            agriculture: AgricultureState::default(),
            resources: ResourceState::default(),
            pollution: PollutionState::default(),
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            time: 0.0,
            population: PopulationState::default(),
            capital: CapitalState::default(),
            agriculture: AgricultureState::default(),
            resources: ResourceState::default(),
            pollution: PollutionState::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Arithmetic for RK4 (operates on the full struct for convenience)
// ---------------------------------------------------------------------------

impl std::ops::Add for WorldState {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.population.cohort_0_14 += rhs.population.cohort_0_14;
        self.population.cohort_15_44 += rhs.population.cohort_15_44;
        self.population.cohort_45_64 += rhs.population.cohort_45_64;
        self.population.cohort_65_plus += rhs.population.cohort_65_plus;
        self.population.perceived_le += rhs.population.perceived_le;
        self.capital.industrial_capital += rhs.capital.industrial_capital;
        self.capital.service_capital += rhs.capital.service_capital;
        self.capital.perceived_iopc += rhs.capital.perceived_iopc;
        self.agriculture.arable_land += rhs.agriculture.arable_land;
        self.agriculture.potentially_arable_land += rhs.agriculture.potentially_arable_land;
        self.agriculture.urban_industrial_land += rhs.agriculture.urban_industrial_land;
        self.agriculture.land_fertility += rhs.agriculture.land_fertility;
        self.agriculture.food_per_capita_smooth += rhs.agriculture.food_per_capita_smooth;
        self.resources.nonrenewable_resources += rhs.resources.nonrenewable_resources;
        self.pollution.persistent_pollution += rhs.pollution.persistent_pollution;
        self.pollution.pollution_appearance_buffer += rhs.pollution.pollution_appearance_buffer;
        self
    }
}

impl std::ops::Mul<f64> for WorldState {
    type Output = Self;
    fn mul(mut self, rhs: f64) -> Self {
        self.population.cohort_0_14 *= rhs;
        self.population.cohort_15_44 *= rhs;
        self.population.cohort_45_64 *= rhs;
        self.population.cohort_65_plus *= rhs;
        self.population.perceived_le *= rhs;
        self.capital.industrial_capital *= rhs;
        self.capital.service_capital *= rhs;
        self.capital.perceived_iopc *= rhs;
        self.agriculture.arable_land *= rhs;
        self.agriculture.potentially_arable_land *= rhs;
        self.agriculture.urban_industrial_land *= rhs;
        self.agriculture.land_fertility *= rhs;
        self.agriculture.food_per_capita_smooth *= rhs;
        self.resources.nonrenewable_resources *= rhs;
        self.pollution.persistent_pollution *= rhs;
        self.pollution.pollution_appearance_buffer *= rhs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_to_vec_from_vec_roundtrip() {
        let s = WorldState::initial_1900();
        let v = s.to_vec();
        let s2 = WorldState::from_vec(s.time, &v);
        let v2 = s2.to_vec();
        for (a, b) in v.iter().zip(v2.iter()) {
            assert_relative_eq!(a, b, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_world_state_n() {
        assert_eq!(WorldState::N, 16);
        let s = WorldState::initial_1900();
        assert_eq!(s.to_vec().len(), WorldState::N);
    }

    #[test]
    fn test_from_vec_clamping() {
        let v = vec![-1.0; WorldState::N];
        let s = WorldState::from_vec(1900.0, &v);
        // Populations clamped to 0
        assert_eq!(s.population.cohort_0_14, 0.0);
        assert_eq!(s.population.cohort_15_44, 0.0);
        assert_eq!(s.population.cohort_45_64, 0.0);
        assert_eq!(s.population.cohort_65_plus, 0.0);
        // Land fertility clamped to 1
        assert_eq!(s.agriculture.land_fertility, 1.0);
        // Perceived LE clamped to 5
        assert_eq!(s.population.perceived_le, 5.0);
        // Resources clamped to 0
        assert_eq!(s.resources.nonrenewable_resources, 0.0);
    }

    #[test]
    fn test_hwi_known_values() {
        let mut s = WorldState::initial_1900();
        // LE=25 → LEI=0 → HWI=0
        s.population.life_expectancy = 25.0;
        s.capital.industrial_output_per_capita = 200.0;
        assert_relative_eq!(s.hwi(), 0.0, epsilon = 1e-10);

        // LE=55, IOPC=$200
        s.population.life_expectancy = 55.0;
        let lei = (55.0 - 25.0) / 60.0; // 0.5
        let ii = (200.0_f64.ln() - 20.0_f64.ln()) / (5000.0_f64.ln() - 20.0_f64.ln());
        let expected = (lei * ii).sqrt();
        assert_relative_eq!(s.hwi(), expected, epsilon = 1e-10);
    }

    #[test]
    fn test_hwi_bounds() {
        let mut s = WorldState::initial_1900();
        // Extreme low
        s.population.life_expectancy = 5.0;
        s.capital.industrial_output_per_capita = 1.0;
        assert!(s.hwi() >= 0.0);
        assert!(s.hwi() <= 1.0);

        // Extreme high
        s.population.life_expectancy = 90.0;
        s.capital.industrial_output_per_capita = 10000.0;
        assert!(s.hwi() >= 0.0);
        assert!(s.hwi() <= 1.0);
    }

    #[test]
    fn test_ecological_footprint_1970() {
        let mut s = WorldState::initial_1900();
        s.agriculture.arable_land = 1.2e9;
        s.agriculture.urban_industrial_land = 50e6;
        s.pollution.pollution_index = 1.0;
        let ef = s.ecological_footprint();
        // EF = (1.2e9 + 50e6 + 1.0 * 0.3e9) / 1.91e9
        let expected = (1.2e9 + 50e6 + 0.3e9) / 1.91e9;
        assert_relative_eq!(ef, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_add_mul_ops() {
        let s = WorldState::initial_1900();
        let double = s.clone() + s.clone();
        let mul2 = s.clone() * 2.0;
        let v_double = double.to_vec();
        let v_mul2 = mul2.to_vec();
        for (a, b) in v_double.iter().zip(v_mul2.iter()) {
            assert_relative_eq!(a, b, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_zero_derivative() {
        let d = WorldState::zero_derivative(1900.0);
        let v = d.to_vec();
        for (i, val) in v.iter().enumerate() {
            assert_eq!(*val, 0.0, "stock {i} should be zero in zero_derivative");
        }
    }

    #[test]
    fn test_initial_1900_population_sum() {
        let s = WorldState::initial_1900();
        let cohort_sum = s.population.cohort_0_14 + s.population.cohort_15_44
            + s.population.cohort_45_64 + s.population.cohort_65_plus;
        assert_relative_eq!(s.population.population, cohort_sum, epsilon = 1e-6);
    }
}
