use std::sync::OnceLock;
use world3_core::{
    model::{params::ScenarioParams, state::WorldState},
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

/// Shared BAU simulation (run once across all tests in this binary).
pub fn bau_sim() -> &'static SimulationOutput {
    static SIM: OnceLock<SimulationOutput> = OnceLock::new();
    SIM.get_or_init(|| {
        let params = ScenarioParams::bau();
        let initial = WorldState::initial_1900();
        let tables = std::sync::Arc::new(
            world3_core::lookup::tables::WorldLookupTables::load(),
        );
        let solver = Rk4Solver::new(tables);
        let states = solver.solve(initial, &params).expect("BAU simulation failed");
        SimulationOutput::new(states, params)
    })
}
