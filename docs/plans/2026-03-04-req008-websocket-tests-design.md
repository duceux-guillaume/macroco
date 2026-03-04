# REQ-008 WebSocket Streaming Simulation — Test Coverage Design

**Date:** 2026-03-04
**Requirement:** REQ-008 (WebSocket streaming simulation)
**Status:** Currently uncovered in traceability matrix

## Goal

Add ~11 tests covering REQ-008 on both backend and frontend, closing one of 4 uncovered Done requirements.

## Backend Integration Tests

**File:** `crates/world3-api/src/routes/ws.rs` (inline `#[cfg(test)]` module)

**New dev dependency:** `tokio-tungstenite` in `crates/world3-api/Cargo.toml`

**Helper:** `async fn spawn_test_server() -> String` — creates `AppState`, builds router, binds to `127.0.0.1:0`, returns the `ws://...` URL.

### Tests

1. **`test_start_simulation_completes`** — Connect via WS, send `StartSimulation` with `scenario_id: "bau"`, assert we receive `SimStep` frames followed by `SimComplete` with correct scenario_id.

2. **`test_start_simulation_with_inline_params`** — Send `StartSimulation` with params field populated (short time range 1900–1910 for speed). Assert `SimComplete` with expected step count.

3. **`test_unknown_scenario_returns_error`** — Send `StartSimulation` with `scenario_id: "nonexistent"`, no params. Assert `SimError` with "not found" message.

4. **`test_invalid_json_returns_error`** — Send malformed JSON text. Assert `SimError` with "Invalid message" text.

5. **`test_update_params_sends_ack`** — Send `UpdateParams` with BAU params. Assert `ParamsAck` with correct scenario_id, then `SimComplete` after debounce.

6. **`test_message_serialization_roundtrip`** — Unit test (no WS needed): serialize `WsClientMsg` variants to JSON, verify tag + fields. Serialize `WsServerMsg` variants, verify structure.

## Frontend Unit Tests

**File:** `frontend/src/lib/ws.test.ts`

**No new dependencies** — mock `WebSocket` global in jsdom.

### Tests

1. **`connectionState transitions on connect`** — Mock WebSocket, call `connect()`, fire `onopen`, assert store goes `disconnected → connecting → connected`.

2. **`onServerMessage dispatches parsed messages`** — Register handler via `onServerMessage()`, fire `onmessage` with `sim_step` JSON, assert handler called with parsed object.

3. **`send serializes and sends when connected`** — Call `send()` with a `WsClientMsg`, assert `WebSocket.send()` called with correct JSON string.

4. **`disconnect cleans up and sets state`** — Call `connect()`, then `disconnect()`, assert `socket.close()` called, state becomes `disconnected`, no reconnect scheduled.

5. **`handler unsubscribe removes handler`** — Register handler, call returned unsubscribe fn, fire message, assert handler NOT called.

## Traceability

- Both test files annotated with `// REQ: REQ-008`
- Traceability matrix regenerated after implementation

## Impact

- Closes 1 of 4 uncovered Done requirements
- Uncovered after: REQ-006, REQ-022, REQ-023
