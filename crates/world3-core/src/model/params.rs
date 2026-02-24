//! Scenario parameters — the "policy levers" exposed as sliders in the UI.
//! Each field has documented units, range, and default value.

use serde::{Deserialize, Serialize};

/// All adjustable parameters for a simulation scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioParams {
    pub meta: ScenarioMeta,

    // ---- Population policy ----
    /// Year at which family planning reaches full effectiveness [1900..2100, default 2000]
    pub family_planning_year: f64,
    /// Maximum effectiveness of family planning [0..1, default 0.75]
    pub family_planning_efficacy: f64,
    /// Health services investment multiplier [0.5..3.0, default 1.0]
    pub health_investment_multiplier: f64,

    // ---- Capital / technology ----
    /// Industrial capital depreciation rate [yr⁻¹, 0.02..0.10, default 0.05]
    pub industrial_depreciation_rate: f64,
    /// Service capital depreciation rate [yr⁻¹, 0.02..0.10, default 0.05]
    pub service_depreciation_rate: f64,
    /// Technology progress rate (TFP growth multiplier) [0..0.03, default 0.002]
    pub technology_growth_rate: f64,
    /// Fraction of industrial output reinvested in industry [0..0.4, default 0.12]
    pub investment_rate: f64,

    // ---- Agriculture ----
    /// Agricultural technology multiplier [0.5..3.0, default 1.0]
    pub agricultural_technology: f64,
    /// Fraction of arable land under protection from degradation [0..0.5, default 0.0]
    pub land_protection_fraction: f64,
    /// Subsistence food threshold [kg/person/yr, default 230.0]
    pub subsistence_food_per_capita: f64,

    // ---- Resources ----
    /// Resource extraction efficiency multiplier [1..5, default 1.0]
    pub resource_efficiency: f64,
    /// Non-renewable resource initial stock [normalized, default 1.0]
    pub initial_nnr_fraction: f64,

    // ---- Pollution ----
    /// Pollution control policy strength [0..1, default 0.0]
    pub pollution_control: f64,

    // ---- Solver configuration ----
    /// Simulation start year [default 1900.0]
    pub start_year: f64,
    /// Simulation end year [default 2100.0]
    pub end_year: f64,
    /// Time step [years, default 1.0]
    pub time_step: f64,
}

impl Default for ScenarioParams {
    fn default() -> Self {
        Self {
            meta: ScenarioMeta::default(),
            family_planning_year: 2000.0,
            family_planning_efficacy: 0.75,
            health_investment_multiplier: 1.0,
            industrial_depreciation_rate: 0.05,
            service_depreciation_rate: 0.05,
            technology_growth_rate: 0.002,
            investment_rate: 0.12,
            agricultural_technology: 1.0,
            land_protection_fraction: 0.0,
            subsistence_food_per_capita: 230.0,
            resource_efficiency: 1.0,
            initial_nnr_fraction: 1.0,
            pollution_control: 0.0,
            start_year: 1900.0,
            end_year: 2100.0,
            time_step: 1.0,
        }
    }
}

/// Business-as-usual scenario (no policy interventions, original World 3 conditions).
impl ScenarioParams {
    pub fn bau() -> Self {
        let mut p = Self::default();
        p.meta.name = "Business as Usual".into();
        p.meta.description = "Original World 3 standard run. No policy interventions.".into();
        p.meta.color_hex = "#e63946".into();
        // BAU has no explicit family planning policy; fertility is purely demand-driven
        // (desired family size responds to rising IOPC through the demographic transition).
        p.family_planning_efficacy = 0.0;
        p
    }

    /// Comprehensive technology scenario — aggressive efficiency gains.
    pub fn comprehensive_technology() -> Self {
        let mut p = Self::default();
        p.meta.name = "Comprehensive Technology".into();
        p.meta.description =
            "Technology solves resource and pollution problems, but no social changes.".into();
        p.meta.color_hex = "#2a9d8f".into();
        p.resource_efficiency = 4.0;
        p.pollution_control = 0.8;
        p.agricultural_technology = 2.0;
        p.technology_growth_rate = 0.02;
        p
    }

    /// Stabilized world scenario — policy + technology + social change.
    pub fn stabilized_world() -> Self {
        let mut p = Self::default();
        p.meta.name = "Stabilized World".into();
        p.meta.description =
            "Combination of technology, pollution control, family planning, and resource efficiency."
                .into();
        p.meta.color_hex = "#457b9d".into();
        p.resource_efficiency = 4.0;
        p.pollution_control = 0.8;
        p.agricultural_technology = 2.0;
        p.technology_growth_rate = 0.015;
        p.family_planning_efficacy = 0.95;
        p.family_planning_year = 1975.0;
        p.land_protection_fraction = 0.3;
        p
    }
}

/// Metadata for a named scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Hex color for chart rendering (e.g. "#e63946")
    pub color_hex: String,
    pub created_at: String,
}

impl Default for ScenarioMeta {
    fn default() -> Self {
        Self {
            id: uuid_v4(),
            name: "Unnamed Scenario".into(),
            description: String::new(),
            color_hex: "#888888".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

fn uuid_v4() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Descriptor for a single parameter — used by the API to generate UI sliders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDescriptor {
    /// Rust field name (snake_case)
    pub field: String,
    /// Human-readable label
    pub label: String,
    /// Unit string (e.g. "yr⁻¹", "kg/person/yr")
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    /// Sector group for UI grouping
    pub sector: String,
    pub description: String,
}

/// Return the full parameter schema for the API.
pub fn parameter_descriptors() -> Vec<ParameterDescriptor> {
    vec![
        ParameterDescriptor {
            field: "family_planning_year".into(),
            label: "Family Planning Year".into(),
            unit: "year".into(),
            min: 1950.0, max: 2100.0, default: 2000.0, step: 5.0,
            sector: "population".into(),
            description: "Year at which family planning reaches full effectiveness.".into(),
        },
        ParameterDescriptor {
            field: "family_planning_efficacy".into(),
            label: "Family Planning Efficacy".into(),
            unit: "fraction".into(),
            min: 0.0, max: 1.0, default: 0.75, step: 0.05,
            sector: "population".into(),
            description: "Maximum reduction in desired family size from family planning programs.".into(),
        },
        ParameterDescriptor {
            field: "health_investment_multiplier".into(),
            label: "Health Investment".into(),
            unit: "multiplier".into(),
            min: 0.5, max: 3.0, default: 1.0, step: 0.1,
            sector: "population".into(),
            description: "Scales health services spending, affecting life expectancy.".into(),
        },
        ParameterDescriptor {
            field: "industrial_depreciation_rate".into(),
            label: "Industrial Capital Depreciation".into(),
            unit: "yr⁻¹".into(),
            min: 0.02, max: 0.10, default: 0.05, step: 0.005,
            sector: "capital".into(),
            description: "Annual fraction of industrial capital that wears out.".into(),
        },
        ParameterDescriptor {
            field: "technology_growth_rate".into(),
            label: "Technology Progress Rate".into(),
            unit: "yr⁻¹".into(),
            min: 0.0, max: 0.03, default: 0.002, step: 0.001,
            sector: "capital".into(),
            description: "Annual improvement in industrial output per unit capital.".into(),
        },
        ParameterDescriptor {
            field: "investment_rate".into(),
            label: "Investment Rate".into(),
            unit: "fraction".into(),
            min: 0.0, max: 0.4, default: 0.12, step: 0.01,
            sector: "capital".into(),
            description: "Fraction of industrial output reinvested in industrial capital.".into(),
        },
        ParameterDescriptor {
            field: "agricultural_technology".into(),
            label: "Agricultural Technology".into(),
            unit: "multiplier".into(),
            min: 0.5, max: 3.0, default: 1.0, step: 0.1,
            sector: "agriculture".into(),
            description: "Multiplier on land yield — represents crop improvements, irrigation.".into(),
        },
        ParameterDescriptor {
            field: "land_protection_fraction".into(),
            label: "Land Protection".into(),
            unit: "fraction".into(),
            min: 0.0, max: 0.5, default: 0.0, step: 0.05,
            sector: "agriculture".into(),
            description: "Fraction of arable land protected from degradation and overuse.".into(),
        },
        ParameterDescriptor {
            field: "resource_efficiency".into(),
            label: "Resource Efficiency".into(),
            unit: "multiplier".into(),
            min: 1.0, max: 5.0, default: 1.0, step: 0.25,
            sector: "resources".into(),
            description: "Reduces resource use per unit of industrial output.".into(),
        },
        ParameterDescriptor {
            field: "pollution_control".into(),
            label: "Pollution Control".into(),
            unit: "fraction".into(),
            min: 0.0, max: 1.0, default: 0.0, step: 0.05,
            sector: "pollution".into(),
            description: "Fraction by which pollution generation is reduced per unit output.".into(),
        },
    ]
}
