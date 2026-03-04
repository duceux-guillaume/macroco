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

        // Apply initial_nnr_fraction parameter to the initial resource stock
        current.resources.nonrenewable_resources = params.initial_nnr_fraction.clamp(0.0, 2.0);
        current.resources.fraction_remaining = current.resources.nonrenewable_resources.clamp(0.0, 1.0);

        // Populate auxiliary fields for the initial state (all sectors, including population)
        let tables = &*self.tables;
        {
            let mut init = current.clone();
            crate::model::sectors::resources::compute_resource_auxiliaries(&mut init, tables);
            crate::model::sectors::capital::capital_derivatives(&mut init, params, tables);
            crate::model::sectors::agriculture::agriculture_derivatives(&mut init, params, tables);
            crate::model::sectors::pollution::pollution_derivatives(&mut init, params, tables);
            crate::model::sectors::population::population_derivatives(&mut init, params, tables);
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
            crate::model::sectors::pollution::pollution_derivatives(&mut next, params, tables);
            crate::model::sectors::population::population_derivatives(&mut next, params, tables);

            // Divergence check — validate key stocks are finite and in-bounds
            let pop = next.population.population;
            if !pop.is_finite() || !(0.0..=1e13).contains(&pop) {
                return Err(SolverError::Diverged {
                    year: next.time,
                    variable: "population".into(),
                    value: pop,
                });
            }
            let ic = next.capital.industrial_capital;
            if !ic.is_finite() {
                return Err(SolverError::Diverged {
                    year: next.time,
                    variable: "industrial_capital".into(),
                    value: ic,
                });
            }
            let pp = next.pollution.persistent_pollution;
            if !pp.is_finite() {
                return Err(SolverError::Diverged {
                    year: next.time,
                    variable: "persistent_pollution".into(),
                    value: pp,
                });
            }

            states.push(next.clone());
            current = next;
        }

        Ok(states)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_solver() -> Rk4Solver {
        let tables = Arc::new(WorldLookupTables::load());
        Rk4Solver::new(tables)
    }

    #[test]
    fn test_bau_simulation_qualitative() {
        let solver = make_solver();
        let params = ScenarioParams::bau();
        let initial = WorldState::initial_1900();
        let states = solver.solve(initial, &params).expect("BAU should not diverge");

        // Should have ~201 steps (1900-2100 inclusive at dt=1)
        assert!(states.len() >= 200);

        // Population should peak and decline
        let (peak_pop, peak_year) = states.iter()
            .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
                if s.population.population > mp {
                    (s.population.population, s.time)
                } else {
                    (mp, my)
                }
            });
        assert!(peak_pop > 4.0e9, "peak pop {:.2e} should exceed 4B", peak_pop);
        assert!(peak_year < 2080.0, "peak year {} should be before 2080", peak_year);

        let final_state = states.last().unwrap();
        assert!(final_state.population.population < peak_pop,
            "2100 pop should be less than peak");

        // Resources should deplete
        assert!(final_state.resources.fraction_remaining < 0.5,
            "NNR should be significantly depleted by 2100");

        // Pollution should have risen above 1.0
        let max_poll = states.iter()
            .map(|s| s.pollution.pollution_index)
            .fold(0.0_f64, f64::max);
        assert!(max_poll > 1.0, "peak pollution {} should exceed 1.0", max_poll);
    }

    #[test]
    fn test_solver_divergence_detection() {
        let solver = make_solver();
        let params = ScenarioParams::bau();
        let mut initial = WorldState::initial_1900();
        // Give absurd initial conditions: population=1e14 (beyond 1e13 threshold)
        initial.population.cohort_0_14 = 1e14;
        initial.population.cohort_15_44 = 1e14;
        initial.population.population = 2e14;

        let result = solver.solve(initial, &params);
        assert!(result.is_err(), "should detect divergence with absurd population");
        if let Err(SolverError::Diverged { variable, .. }) = result {
            assert_eq!(variable, "population");
        }
    }

    #[test]
    fn test_short_simulation() {
        let solver = make_solver();
        let mut params = ScenarioParams::bau();
        params.start_year = 1900.0;
        params.end_year = 1910.0;
        let initial = WorldState::initial_1900();
        let states = solver.solve(initial, &params).expect("short sim should succeed");
        assert_eq!(states.len(), 11); // 1900..=1910
        // Population should grow over 10 years
        assert!(states.last().unwrap().population.population
            > states.first().unwrap().population.population);
    }
}
