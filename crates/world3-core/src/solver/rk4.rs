//! 4th-order Runge-Kutta solver for the World 3 ODE system.
//!
//! The RK4 method provides 4th-order accuracy with 4 derivative evaluations
//! per time step. For a 300-year simulation at Δt = 1.0 year, this means
//! 1200 derivative evaluations total.
//!
//! After each step, auxiliary fields are recomputed on the accepted state
//! so the stored trajectory has fully populated `food_per_capita`,
//! `industrial_output`, `pollution_index`, etc.

use crate::lookup::tables::WorldLookupTables;
use crate::model::{
    derivatives::derivatives,
    params::ScenarioParams,
    state::WorldState,
};
use crate::solver::traits::{OdeSolver, SolverError};

pub struct Rk4Solver {
    pub tables: std::sync::Arc<WorldLookupTables>,
}

impl Rk4Solver {
    pub fn new(tables: std::sync::Arc<WorldLookupTables>) -> Self {
        Self { tables }
    }

    fn rk4_step(
        &self,
        state: &WorldState,
        dt: f64,
        params: &ScenarioParams,
    ) -> WorldState {
        let tables = &*self.tables;

        // k1 = f(t, y)
        // `derivatives()` recomputes all auxiliaries from stock values — no pre-population needed
        let k1 = derivatives(state, params, tables);

        // k2 = f(t + dt/2, y + k1*dt/2)
        let s2_stocks = state.clone() + k1.clone() * (dt / 2.0);
        let mut s2 = WorldState::from_vec(state.time + dt / 2.0, &s2_stocks.to_vec());
        s2.time = state.time + dt / 2.0;
        let k2 = derivatives(&s2, params, tables);

        // k3 = f(t + dt/2, y + k2*dt/2)
        let s3_stocks = state.clone() + k2.clone() * (dt / 2.0);
        let mut s3 = WorldState::from_vec(state.time + dt / 2.0, &s3_stocks.to_vec());
        s3.time = state.time + dt / 2.0;
        let k3 = derivatives(&s3, params, tables);

        // k4 = f(t + dt, y + k3*dt)
        let s4_stocks = state.clone() + k3.clone() * dt;
        let mut s4 = WorldState::from_vec(state.time + dt, &s4_stocks.to_vec());
        s4.time = state.time + dt;
        let k4 = derivatives(&s4, params, tables);

        // Weighted sum: y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let weighted = k1 + k2.clone() * 2.0 + k3.clone() * 2.0 + k4;
        let new_state = state.clone() + weighted * (dt / 6.0);

        // Reconstruct from vec to apply clamping (no negative populations etc.)
        let mut result = WorldState::from_vec(state.time + dt, &new_state.to_vec());
        result.time = state.time + dt;
        result
    }
}

impl OdeSolver for Rk4Solver {
    fn solve(
        &self,
        initial: WorldState,
        params: &ScenarioParams,
    ) -> Result<Vec<WorldState>, SolverError> {
        let dt = params.time_step;
        let n_steps =
            ((params.end_year - params.start_year) / dt).ceil() as usize + 1;

        let mut states = Vec::with_capacity(n_steps);
        let mut current = initial;

        // Populate auxiliary fields for the initial state
        let tables = &*self.tables;
        {
            let mut init = current.clone();
            crate::model::sectors::resources::compute_resource_auxiliaries(&mut init, tables);
            crate::model::sectors::capital::capital_derivatives(&mut init, params, tables);
            crate::model::sectors::agriculture::agriculture_derivatives(&mut init, params, tables);
            crate::model::sectors::pollution::pollution_derivative(&mut init, params, tables);
            current = init;
        }

        states.push(current.clone());

        while current.time < params.end_year - dt * 0.5 {
            let actual_dt = if current.time + dt > params.end_year {
                params.end_year - current.time
            } else {
                dt
            };

            let mut next = self.rk4_step(&current, actual_dt, params);

            // Recompute all auxiliary fields on the accepted state
            crate::model::sectors::resources::compute_resource_auxiliaries(&mut next, tables);
            crate::model::sectors::capital::capital_derivatives(&mut next, params, tables);
            crate::model::sectors::agriculture::agriculture_derivatives(&mut next, params, tables);
            crate::model::sectors::pollution::pollution_derivative(&mut next, params, tables);
            crate::model::sectors::population::population_derivatives(&mut next, params, tables);

            // Divergence check
            let pop = next.population.population;
            if !pop.is_finite() || pop < 0.0 || pop > 1e13 {
                return Err(SolverError::Diverged {
                    year: next.time,
                    variable: "population".into(),
                    value: pop,
                });
            }

            states.push(next.clone());
            current = next;
        }

        Ok(states)
    }
}
