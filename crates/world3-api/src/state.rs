use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, RwLock};
use world3_core::{
    lookup::tables::WorldLookupTables, Rk4Solver, ScenarioParams,
};

use crate::models::Scenario;

pub struct AppState {
    pub solver: Arc<Rk4Solver>,
    pub scenarios: Arc<RwLock<HashMap<String, Scenario>>>,
    /// Phase 4 placeholder — ingestion broadcast
    pub _ingestion_tx: broadcast::Sender<()>,
}

pub fn init_app_state() -> AppState {
    // 1. Load lookup tables and build solver
    let tables = Arc::new(WorldLookupTables::load());
    let solver = Arc::new(Rk4Solver::new(tables));

    // 2. Pre-populate with 3 preset scenarios
    let presets: Vec<ScenarioParams> = vec![
        ScenarioParams::bau(),
        ScenarioParams::comprehensive_technology(),
        ScenarioParams::stabilized_world(),
    ];

    let mut map: HashMap<String, Scenario> = HashMap::new();
    for params in presets {
        let id = params.meta.id.clone();
        map.insert(
            id,
            Scenario {
                params,
                is_preset: true,
                last_output: None,
            },
        );
    }

    // 3. Phase 4 placeholder broadcast channel (capacity 1)
    let (tx, _) = broadcast::channel(1);

    AppState {
        solver,
        scenarios: Arc::new(RwLock::new(map)),
        _ingestion_tx: tx,
    }
}
