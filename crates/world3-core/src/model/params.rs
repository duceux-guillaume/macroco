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
    /// Maximum effectiveness of family planning [0..1, default 0.0 (BAU)]
    pub family_planning_efficacy: f64,
    /// Health services investment multiplier [0.5..3.0, default 1.0]
    pub health_investment_multiplier: f64,

    // ---- Capital / technology ----
    /// Industrial capital depreciation rate [yr⁻¹]. World3-03: 1/alic1 = 1/13 ≈ 0.0769
    pub industrial_depreciation_rate: f64,
    /// Service capital depreciation rate [yr⁻¹]. World3-03: 1/alsc1 = 1/20 = 0.05
    pub service_depreciation_rate: f64,
    /// Technology progress rate (TFP growth multiplier) [0..0.03, default 0.002]
    pub technology_growth_rate: f64,

    // ---- Agriculture ----
    /// Agricultural technology multiplier [0.5..3.0, default 1.0]
    pub agricultural_technology: f64,
    /// Fraction of arable land under protection from degradation [0..0.5, default 0.0]
    pub land_protection_fraction: f64,
    /// Subsistence food threshold [kg/person/yr, default 230.0]
    pub subsistence_food_per_capita: f64,
    /// Agricultural technology growth rate [yr⁻¹, default 0.005].
    /// Macroco extension: represents Green Revolution TFP not captured by LYMC.
    /// Source: USDA ERS International Agricultural Productivity (~1%/yr, 1960-2020).
    /// Applied from 1960: ag_tech = agricultural_technology × (1 + rate)^max(year-1960, 0).
    pub agricultural_technology_growth_rate: f64,

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
            // Default matches BAU: no family planning intervention
            family_planning_efficacy: 0.0,
            health_investment_multiplier: 1.0,
            // World3-03: alic1 = 14 yr → depreciation = 1/14.
            // Tuned to 1/13 to reduce early IOPC overshoot (LE structural changes
            // increased population growth, pushing IOPC too high in 1960).
            industrial_depreciation_rate: 1.0 / 13.0,
            // World3-03: alsc1 = 20 yr → depreciation = 1/20
            service_depreciation_rate: 0.05,
            // Calibrated to match historical IOPC trajectory (World Bank 1960-2023).
            // Higher than original 0.002 because ISOPC dynamic lookup captures
            // service-capital feedback; tech rate handles remaining industrial
            // productivity growth not in World3-03 structure (~1.5%/yr TFP).
            technology_growth_rate: 0.014,
            agricultural_technology: 1.0,
            // Macroco extension: agricultural TFP growth from Green Revolution.
            // USDA ERS data shows ~1%/yr globally (1960-2020). Set to 0.005
            // because LYMC already captures input-driven yield gains; this is
            // the residual TFP (improved cultivars, practices, irrigation tech).
            agricultural_technology_growth_rate: 0.005,
            land_protection_fraction: 0.0,
            subsistence_food_per_capita: 230.0,
            // Slightly above 1.0 to account for real-world resource extraction
            // efficiency gains not modeled in World3-03. Helps IOPC calibration
            // (delays NNR depletion to match historical trajectory).
            resource_efficiency: 1.05,
            initial_nnr_fraction: 1.0,
            pollution_control: 0.0,
            start_year: 1900.0,
            end_year: 2100.0,
            time_step: 1.0,
        }
    }
}

/// Business-as-usual scenario (no policy interventions, original World 3 conditions).
/// Default now matches BAU so that API consumers using default() get BAU behavior.
impl ScenarioParams {
    pub fn bau() -> Self {
        let mut p = Self::default();
        p.meta.name = "Business as Usual".into();
        p.meta.description = "Original World 3 standard run. No policy interventions.".into();
        p.meta.color_hex = "#e63946".into();
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
        p.agricultural_technology_growth_rate = 0.0;
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
        p.agricultural_technology_growth_rate = 0.0;
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
            id: scenario_id(),
            name: "Unnamed Scenario".into(),
            description: String::new(),
            color_hex: "#888888".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Generate a short unique scenario ID from system time.
/// Not a real UUID — just a 16-hex-char hash for local identification.
fn scenario_id() -> String {
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
            min: 0.0, max: 1.0, default: 0.0, step: 0.05,
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
            min: 0.02, max: 0.15, default: 1.0 / 13.0, step: 0.005,
            sector: "capital".into(),
            description: "Annual fraction of industrial capital that wears out.".into(),
        },
        ParameterDescriptor {
            field: "technology_growth_rate".into(),
            label: "Technology Progress Rate".into(),
            unit: "yr⁻¹".into(),
            min: 0.0, max: 0.03, default: 0.014, step: 0.001,
            sector: "capital".into(),
            description: "Annual improvement in industrial output per unit capital.".into(),
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
            field: "agricultural_technology_growth_rate".into(),
            label: "Agricultural Tech Growth".into(),
            unit: "yr⁻¹".into(),
            min: 0.0, max: 0.02, default: 0.005, step: 0.001,
            sector: "agriculture".into(),
            description: "Annual improvement in agricultural yield from Green Revolution and modern farming advances. Macroco extension beyond World3-03.".into(),
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
            min: 1.0, max: 5.0, default: 1.05, step: 0.25,
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
        ParameterDescriptor {
            field: "service_depreciation_rate".into(),
            label: "Service Depreciation".into(),
            unit: "yr⁻¹".into(),
            min: 0.02, max: 0.15, default: 0.05, step: 0.005,
            sector: "capital".into(),
            description: "Annual fraction of service capital (hospitals, schools) that wears out.".into(),
        },
        ParameterDescriptor {
            field: "subsistence_food_per_capita".into(),
            label: "Subsistence Food Level".into(),
            unit: "kg/person/yr".into(),
            min: 150.0, max: 350.0, default: 230.0, step: 10.0,
            sector: "agriculture".into(),
            description: "Minimum food per person for basic health. Below this, life expectancy falls.".into(),
        },
        ParameterDescriptor {
            field: "initial_nnr_fraction".into(),
            label: "Initial Resource Level".into(),
            unit: "fraction".into(),
            min: 0.25, max: 2.0, default: 1.0, step: 0.25,
            sector: "resources".into(),
            description: "Starting level of non-renewable resources (1.0 = full endowment).".into(),
        },
    ]
}

// REQ: REQ-004
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_matches_bau() {
        let d = ScenarioParams::default();
        let b = ScenarioParams::bau();
        // Critical fields must match (meta differs)
        assert_eq!(d.family_planning_efficacy, b.family_planning_efficacy);
        assert_eq!(d.industrial_depreciation_rate, b.industrial_depreciation_rate);
        assert_eq!(d.service_depreciation_rate, b.service_depreciation_rate);
        assert_eq!(d.technology_growth_rate, b.technology_growth_rate);
        assert_eq!(d.resource_efficiency, b.resource_efficiency);
        assert_eq!(d.pollution_control, b.pollution_control);
        assert_eq!(d.agricultural_technology, b.agricultural_technology);
        assert_eq!(d.agricultural_technology_growth_rate, b.agricultural_technology_growth_rate);
    }

    #[test]
    fn test_presets_valid_ranges() {
        let descriptors = parameter_descriptors();
        let presets = [
            ScenarioParams::bau(),
            ScenarioParams::comprehensive_technology(),
            ScenarioParams::stabilized_world(),
        ];
        for preset in &presets {
            for desc in &descriptors {
                let val = match desc.field.as_str() {
                    "family_planning_year" => preset.family_planning_year,
                    "family_planning_efficacy" => preset.family_planning_efficacy,
                    "health_investment_multiplier" => preset.health_investment_multiplier,
                    "industrial_depreciation_rate" => preset.industrial_depreciation_rate,
                    "technology_growth_rate" => preset.technology_growth_rate,
                    "agricultural_technology" => preset.agricultural_technology,
                    "agricultural_technology_growth_rate" => preset.agricultural_technology_growth_rate,
                    "land_protection_fraction" => preset.land_protection_fraction,
                    "resource_efficiency" => preset.resource_efficiency,
                    "pollution_control" => preset.pollution_control,
                    _ => continue,
                };
                assert!(
                    val >= desc.min && val <= desc.max,
                    "Preset '{}' field '{}': value {} outside [{}, {}]",
                    preset.meta.name, desc.field, val, desc.min, desc.max
                );
            }
        }
    }
}
