//! Non-renewable resource sector.
//!
//! Resources are depleted by industrial activity. As the fraction remaining
//! falls, the cost multiplier rises, diverting ever-larger shares of industrial
//! capital to resource extraction instead of productive output.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{params::ScenarioParams, state::WorldState};

/// Resource depletion coefficient [NNR_fraction / (person × USD/person/yr × year)].
///
/// World3-03: NRI = 1e12 resource units, NRUR at 1970 ≈ 1e10 units/yr.
/// With IOPC ≈ $200/person/yr, POP ≈ 3.6e9:
///   coeff = 1e10 / (3.6e9 × 200 × 1e12) = 1.39e-14
/// Calibrated to deplete ~50% of NNR by 2030 in BAU.
const RESOURCE_DEPLETION_COEFF: f64 = 1.4e-14;

/// Compute the rate of change of non-renewable resources.
///
/// Returns `d(nonrenewable_resources)/dt` in resource units per year.
/// This is always negative (resources are consumed, never replenished).
///
/// Mechanism:
/// - Per-capita resource use scales with industrial output per capita (IOPC).
/// - Total extraction = POP × per_capita_use / resource_efficiency.
/// - The FCAOR table (capital fraction for resource extraction) is used ONLY by
///   the capital sector to reduce productive output — it does NOT appear here.
pub fn resource_derivative(
    state: &WorldState,
    params: &ScenarioParams,
    _tables: &WorldLookupTables,
) -> f64 {
    let pop = state.population.population;
    if pop <= 0.0 {
        return 0.0;
    }

    // Per-capita resource demand scales with industrial output per capita
    let iopc = state.capital.industrial_output_per_capita.max(0.0);

    // Extraction rate = POP × IOPC × coefficient / efficiency
    // Decreases naturally as resources deplete (via feedback through capital output)
    let extraction_rate = pop * iopc * RESOURCE_DEPLETION_COEFF / params.resource_efficiency;

    -extraction_rate
}

/// Compute auxiliary variables for the resource sector.
///
/// Updates `state.resources.fraction_remaining` in place.
/// Must be called before capital sector uses the cost multiplier.
pub fn compute_resource_auxiliaries(state: &mut WorldState, tables: &WorldLookupTables) {
    state.resources.fraction_remaining =
        state.resources.nonrenewable_resources.clamp(0.0, 1.0);

    // Capital-output ratio multiplier is stored on capital sector (capital.rs computes it)
    let _ = tables
        .capital_output_ratio_resources
        .eval(state.resources.fraction_remaining);
}
