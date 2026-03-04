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
    let start_year = params.start_year;
    tokio::spawn(async move {
        let solver = Arc::clone(&state.solver);
        let initial = initial_conditions_1900();

        let result = tokio::task::spawn_blocking(move || solver.solve(initial, &params)).await;

        match result {
            Ok(Ok(states)) => {
                let mut n = 0usize;
                for s in states {
                    if s.time < start_year {
                        continue;
                    }
                    n += 1;
                    let year = s.time;
                    if tx
                        .send(WsServerMsg::SimStep { year, state: Box::new(s) })
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

// REQ: REQ-008
#[cfg(test)]
mod tests {
    use crate::models::{WsClientMsg, WsServerMsg};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{connect_async, tungstenite};

    #[test]
    fn test_message_serialization_roundtrip() {
        // WsClientMsg variants serialize with correct tags
        let start = WsClientMsg::StartSimulation {
            scenario_id: "bau".into(),
            params: None,
        };
        let json = serde_json::to_string(&start).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "start_simulation");
        assert_eq!(v["scenario_id"], "bau");
        assert!(v.get("params").is_some());

        let stop = WsClientMsg::StopSimulation;
        let json = serde_json::to_string(&stop).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "stop_simulation");

        // WsServerMsg variants
        let err = WsServerMsg::SimError {
            message: "boom".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "sim_error");
        assert_eq!(v["message"], "boom");

        let ack = WsServerMsg::ParamsAck {
            scenario_id: "bau".into(),
        };
        let json = serde_json::to_string(&ack).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "params_ack");

        let complete = WsServerMsg::SimComplete {
            scenario_id: "bau".into(),
            total_steps: 42,
        };
        let json = serde_json::to_string(&complete).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "sim_complete");
        assert_eq!(v["total_steps"], 42);
    }

    /// Spawn the full Axum app on a random port, return the WS URL.
    async fn spawn_test_server() -> String {
        let state = crate::state::init_app_state();
        let app = crate::routes::build_router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("ws://{}/api/v1/ws", addr)
    }

    /// Read the next text message from the WS stream, parse as JSON Value.
    async fn recv_json(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> serde_json::Value {
        use futures_util::StreamExt;
        loop {
            let msg = ws.next().await.unwrap().unwrap();
            if let tungstenite::Message::Text(text) = msg {
                return serde_json::from_str(&text).unwrap();
            }
        }
    }

    /// Send a JSON text message.
    async fn send_json(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        value: &impl serde::Serialize,
    ) {
        use futures_util::SinkExt;
        let text = serde_json::to_string(value).unwrap();
        ws.send(tungstenite::Message::Text(text)).await.unwrap();
    }

    #[tokio::test]
    async fn test_start_simulation_completes() {
        let url = spawn_test_server().await;
        let (mut ws, _) = connect_async(&url).await.unwrap();

        let mut params = world3_core::ScenarioParams::bau();
        params.end_year = 1910.0;
        let msg = WsClientMsg::StartSimulation {
            scenario_id: "bau".into(),
            params: Some(params),
        };
        send_json(&mut ws, &msg).await;

        let mut step_count = 0usize;
        loop {
            let v = recv_json(&mut ws).await;
            match v["type"].as_str().unwrap() {
                "sim_step" => {
                    step_count += 1;
                    assert!(v["year"].as_f64().is_some());
                    assert!(v["state"].is_object());
                }
                "sim_complete" => {
                    assert_eq!(v["scenario_id"], "bau");
                    assert!(v["total_steps"].as_u64().unwrap() > 0);
                    break;
                }
                "sim_error" => panic!("Unexpected error: {}", v["message"]),
                other => panic!("Unexpected message type: {other}"),
            }
        }
        assert!(step_count > 0, "Should have received sim_step frames");
    }

    #[tokio::test]
    async fn test_start_simulation_with_inline_params() {
        let url = spawn_test_server().await;
        let (mut ws, _) = connect_async(&url).await.unwrap();

        let mut params = world3_core::ScenarioParams::bau();
        params.end_year = 1910.0;
        let msg = WsClientMsg::StartSimulation {
            scenario_id: "custom".into(),
            params: Some(params),
        };
        send_json(&mut ws, &msg).await;

        let mut step_count = 0usize;
        loop {
            let v = recv_json(&mut ws).await;
            match v["type"].as_str().unwrap() {
                "sim_step" => step_count += 1,
                "sim_complete" => {
                    assert_eq!(v["scenario_id"], "custom");
                    assert_eq!(v["total_steps"].as_u64().unwrap(), step_count as u64);
                    break;
                }
                other => panic!("Unexpected: {other}"),
            }
        }
        assert!(step_count > 0);
        assert!(step_count <= 12, "Expected ~10 steps, got {step_count}");
    }

    #[tokio::test]
    async fn test_unknown_scenario_returns_error() {
        let url = spawn_test_server().await;
        let (mut ws, _) = connect_async(&url).await.unwrap();

        let msg = WsClientMsg::StartSimulation {
            scenario_id: "nonexistent".into(),
            params: None,
        };
        send_json(&mut ws, &msg).await;

        let v = recv_json(&mut ws).await;
        assert_eq!(v["type"], "sim_error");
        let message = v["message"].as_str().unwrap();
        assert!(
            message.contains("not found"),
            "Expected 'not found' in error, got: {message}"
        );
    }

    #[tokio::test]
    async fn test_invalid_json_returns_error() {
        let url = spawn_test_server().await;
        let (mut ws, _) = connect_async(&url).await.unwrap();

        use futures_util::SinkExt;
        ws.send(tungstenite::Message::Text("not valid json {{{".into()))
            .await
            .unwrap();

        let v = recv_json(&mut ws).await;
        assert_eq!(v["type"], "sim_error");
        let message = v["message"].as_str().unwrap();
        assert!(
            message.contains("Invalid message"),
            "Expected 'Invalid message' in error, got: {message}"
        );
    }

    #[tokio::test]
    async fn test_update_params_sends_ack() {
        let url = spawn_test_server().await;
        let (mut ws, _) = connect_async(&url).await.unwrap();

        let mut short_params = world3_core::ScenarioParams::bau();
        short_params.end_year = 1910.0;

        let msg = WsClientMsg::UpdateParams {
            scenario_id: "bau".into(),
            params: short_params,
        };
        send_json(&mut ws, &msg).await;

        let v = recv_json(&mut ws).await;
        assert_eq!(v["type"], "params_ack");
        assert_eq!(v["scenario_id"], "bau");

        loop {
            let v = recv_json(&mut ws).await;
            match v["type"].as_str().unwrap() {
                "sim_step" => continue,
                "sim_complete" => {
                    assert_eq!(v["scenario_id"], "bau");
                    break;
                }
                other => panic!("Unexpected: {other}"),
            }
        }
    }
}
