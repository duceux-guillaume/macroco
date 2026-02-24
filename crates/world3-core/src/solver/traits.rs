use crate::model::{params::ScenarioParams, state::WorldState};

/// Abstract ODE solver.
pub trait OdeSolver: Send + Sync {
    /// Integrate the World 3 ODE from `initial.time` to `params.end_year`
    /// in steps of `params.time_step`, returning one `WorldState` per step.
    fn solve(
        &self,
        initial: WorldState,
        params: &ScenarioParams,
    ) -> Result<Vec<WorldState>, SolverError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("State diverged at year {year:.1}: {variable} = {value:.3e}")]
    Diverged {
        year: f64,
        variable: String,
        value: f64,
    },
    #[error("Invalid initial conditions: {0}")]
    InvalidInitial(String),
}
