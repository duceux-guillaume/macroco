use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, Duration},
};
use world3_core::solver::traits::OdeSolver;

use crate::{
    models::{initial_conditions_1900, WsClientMsg, WsServerMsg},
    state::AppState,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let (tx, mut rx) = mpsc::channel::<WsServerMsg>(256);
    let mut sim_task: Option<JoinHandle<()>> = None;
    let mut debounce_task: Option<JoinHandle<()>> = None;

    loop {
        tokio::select! {
            // Incoming WS message from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsClientMsg>(&text) {
                            Ok(client_msg) => {
                                handle_client_msg(
                                    client_msg,
                                    &state,
                                    &tx,
                                    &mut sim_task,
                                    &mut debounce_task,
                                ).await;
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(WsServerMsg::SimError {
                                        message: format!("Invalid message: {}", e),
                                    })
                                    .await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // Client disconnected
                        break;
                    }
                    Some(Ok(_)) => {} // Ignore binary/ping/pong
                    Some(Err(_)) => break,
                }
            }

            // Outbound message from simulation task
            Some(server_msg) = rx.recv() => {
                let json = match serde_json::to_string(&server_msg) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    }

    // Clean up
    if let Some(t) = sim_task {
        t.abort();
    }
    if let Some(t) = debounce_task {
        t.abort();
    }
}

async fn handle_client_msg(
    msg: WsClientMsg,
    state: &Arc<AppState>,
    tx: &mpsc::Sender<WsServerMsg>,
    sim_task: &mut Option<JoinHandle<()>>,
    debounce_task: &mut Option<JoinHandle<()>>,
) {
    match msg {
        WsClientMsg::StartSimulation { scenario_id, params } => {
            // Abort existing tasks
            if let Some(t) = sim_task.take() {
                t.abort();
            }
            if let Some(t) = debounce_task.take() {
                t.abort();
            }

            // Resolve params: use provided override or load from store
            let resolved_params = if let Some(p) = params {
                p
            } else {
                let store = state.scenarios.read().await;
                match store.get(&scenario_id).map(|s| s.params.clone()) {
                    Some(p) => p,
                    None => {
                        let _ = tx
                            .send(WsServerMsg::SimError {
                                message: format!("Scenario '{}' not found", scenario_id),
                            })
                            .await;
                        return;
                    }
                }
            };

            *sim_task = Some(spawn_sim_task(
                Arc::clone(state),
                scenario_id,
                resolved_params,
                tx.clone(),
            ));
        }

        WsClientMsg::UpdateParams { scenario_id, params } => {
            // Abort existing
            if let Some(t) = sim_task.take() {
                t.abort();
            }
            if let Some(t) = debounce_task.take() {
                t.abort();
            }

            // Store updated params
            {
                let mut store = state.scenarios.write().await;
                if let Some(s) = store.get_mut(&scenario_id) {
                    s.params = params.clone();
                    s.last_output = None;
                }
            }

            let _ = tx
                .send(WsServerMsg::ParamsAck {
                    scenario_id: scenario_id.clone(),
                })
                .await;

            // Debounce: wait 50ms then launch sim
            let state2 = Arc::clone(state);
            let tx2 = tx.clone();
            let sid = scenario_id.clone();
            let p = params;

            let debounce = tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                drop(spawn_sim_task(state2, sid, p, tx2));
            });
            *debounce_task = Some(debounce);
        }

        WsClientMsg::StopSimulation => {
            if let Some(t) = sim_task.take() {
                t.abort();
            }
            if let Some(t) = debounce_task.take() {
                t.abort();
            }
        }
    }
}

fn spawn_sim_task(
    state: Arc<AppState>,
    scenario_id: String,
    params: world3_core::ScenarioParams,
    tx: mpsc::Sender<WsServerMsg>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let solver = Arc::clone(&state.solver);
        let initial = initial_conditions_1900();

        let result = tokio::task::spawn_blocking(move || solver.solve(initial, &params)).await;

        match result {
            Ok(Ok(states)) => {
                let n = states.len();
                for s in states {
                    let year = s.time;
                    if tx
                        .send(WsServerMsg::SimStep { year, state: s })
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                let _ = tx
                    .send(WsServerMsg::SimComplete {
                        scenario_id,
                        total_steps: n,
                    })
                    .await;
            }
            Ok(Err(e)) => {
                let _ = tx
                    .send(WsServerMsg::SimError {
                        message: e.to_string(),
                    })
                    .await;
            }
            Err(_) => {} // Task was aborted — normal
        }
    })
}
