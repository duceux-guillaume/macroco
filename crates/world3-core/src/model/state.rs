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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapitalState {
    /// Industrial capital stock [1975 USD]
    pub industrial_capital: f64,
    /// Service capital stock [1975 USD]
    pub service_capital: f64,
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
    /// Annual food production [vegetable-equivalent kg / year]
    pub food: f64,
    /// Food per capita [kg / person / year]
    pub food_per_capita: f64,
    /// Land yield [kg / hectare / year]
    pub land_yield: f64,
    /// Agricultural capital inputs [1975 USD / hectare / year]
    pub agricultural_inputs_per_hectare: f64,
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
    /// Persistent pollution stock [pollution units, 1970 = 1]
    pub persistent_pollution: f64,
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
    pub const N: usize = 10;

    /// Extract the integrable state variables into a flat `Vec<f64>`.
    /// `time` is not included — the solver manages time separately.
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            // Population (4 cohorts — total is derived)
            self.population.cohort_0_14,
            self.population.cohort_15_44,
            self.population.cohort_45_64,
            self.population.cohort_65_plus,
            // Capital (2 stocks)
            self.capital.industrial_capital,
            self.capital.service_capital,
            // Agriculture (2 stocks)
            self.agriculture.arable_land,
            self.agriculture.potentially_arable_land,
            // Resources (1 stock)
            self.resources.nonrenewable_resources,
            // Pollution (1 stock)
            self.pollution.persistent_pollution,
        ]
    }

    /// Reconstruct state from a flat vec (only the 10 ODE stocks).
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

        s.agriculture.arable_land = v[6].max(0.0);
        s.agriculture.potentially_arable_land = v[7].max(0.0);

        s.resources.nonrenewable_resources = v[8].max(0.0);
        s.resources.fraction_remaining = v[8].clamp(0.0, 1.0);

        s.pollution.persistent_pollution = v[9].max(0.0);
        s
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
        self.capital.industrial_capital += rhs.capital.industrial_capital;
        self.capital.service_capital += rhs.capital.service_capital;
        self.agriculture.arable_land += rhs.agriculture.arable_land;
        self.agriculture.potentially_arable_land += rhs.agriculture.potentially_arable_land;
        self.resources.nonrenewable_resources += rhs.resources.nonrenewable_resources;
        self.pollution.persistent_pollution += rhs.pollution.persistent_pollution;
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
        self.capital.industrial_capital *= rhs;
        self.capital.service_capital *= rhs;
        self.agriculture.arable_land *= rhs;
        self.agriculture.potentially_arable_land *= rhs;
        self.resources.nonrenewable_resources *= rhs;
        self.pollution.persistent_pollution *= rhs;
        self
    }
}
