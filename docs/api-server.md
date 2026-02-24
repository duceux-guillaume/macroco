# API Server

The `world3-api` crate provides an Axum HTTP + WebSocket server for running simulations and managing scenarios.

## Starting the Server

```bash
RUST_LOG=info cargo run --bin world3-api
# Listens on 0.0.0.0:8080 by default
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | TCP port to bind |
| `RUST_LOG` | `info` | Log level (e.g. `debug`, `info,world3_api=debug`) |

## Architecture

- **`AppState`** holds: the RK4 solver (`Arc<Rk4Solver>`), a scenario store (`Arc<RwLock<HashMap<String, Scenario>>>`), and a broadcast sender (Phase 4 placeholder)
- All simulations run via `tokio::task::spawn_blocking()` to avoid blocking the async reactor
- CORS is permissive (any origin, any method, any header)
- Three preset scenarios (BAU, Technology, Stabilized) are pre-populated at startup

## REST Endpoints

All endpoints are under `/api/v1`:

### Health & Metadata

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `{"status":"ok","version":"0.1.0"}` |
| GET | `/params/schema` | Parameter descriptors for UI sliders |
| GET | `/presets` | List the 3 built-in preset scenarios |

### Scenario CRUD

| Method | Path | Description |
|--------|------|-------------|
| GET | `/scenarios` | List all scenarios (presets + user-created) |
| POST | `/scenarios` | Create a new scenario from `ScenarioParams` body |
| GET | `/scenarios/:id` | Get a scenario with its last simulation output |
| PUT | `/scenarios/:id/params` | Replace a scenario's parameters (clears cached output) |
| DELETE | `/scenarios/:id` | Delete a scenario (403 if preset) |

### Simulation

| Method | Path | Description |
|--------|------|-------------|
| POST | `/scenarios/:id/run` | Run a full simulation; returns `SimulationOutput` JSON |
| GET | `/ws` | WebSocket upgrade endpoint |

> **Note:** Preset IDs are generated at server startup (time-based hash) and change on each restart. Always call `/presets` to discover current IDs.

### curl Examples

```bash
# Health check
curl http://localhost:8080/api/v1/health

# List presets (get IDs)
curl http://localhost:8080/api/v1/presets

# Run a preset simulation
curl -s -X POST http://localhost:8080/api/v1/scenarios/<ID>/run | jq '.states | length'

# Create a custom scenario
curl -s -X POST http://localhost:8080/api/v1/scenarios \
  -H 'Content-Type: application/json' \
  -d '{"name":"My scenario","params":{"resource_technology_factor":2.0}}' | jq .

# Delete a custom scenario
curl -s -X DELETE http://localhost:8080/api/v1/scenarios/<ID>
```

## WebSocket Protocol

Connect to `ws://localhost:8080/api/v1/ws`.

### Flow

```
Client                              Server
  │                                    │
  │─── start_simulation ──────────────▶│
  │                                    │ spawn_blocking(solver.solve())
  │◀── sim_step {year:1900, state} ────│
  │◀── sim_step {year:1901, state} ────│
  │           ...×201...               │
  │◀── sim_complete {total_steps} ─────│
  │                                    │
  │─── update_params ─────────────────▶│
  │◀── params_ack ─────────────────────│
  │           (50ms debounce)          │
  │◀── sim_step ... ──────────────────│
  │◀── sim_complete ──────────────────│
```

### Client Messages (`WsClientMsg`)

| `"type"` | Fields | Description |
|----------|--------|-------------|
| `start_simulation` | `scenario_id`, optional `params` | Start (or restart) a simulation. If `params` is provided, overrides stored scenario. |
| `update_params` | `scenario_id`, `params` | Update scenario parameters. Server sends `params_ack` immediately, then restarts simulation after 50ms debounce. |
| `stop_simulation` | _(none)_ | Halt current simulation and cancel pending debounce. |

### Server Messages (`WsServerMsg`)

| `"type"` | Fields | Description |
|----------|--------|-------------|
| `sim_step` | `year`, `state` | One message per integration step (e.g. 201 messages for 1900-2100). |
| `sim_complete` | `scenario_id`, `total_steps` | Emitted after the final step. |
| `sim_error` | `message` | Emitted on solver failure (e.g. divergence). |
| `params_ack` | `scenario_id` | Immediate acknowledgement of `update_params`. |

### Debounce Behavior

When `update_params` is received:
1. Server writes new params to the scenario store
2. Server sends `params_ack` immediately
3. Any existing simulation and pending debounce tasks are aborted
4. A new 50ms debounce timer starts
5. After 50ms with no further updates, the simulation restarts automatically

This prevents excessive restarts when the user drags a slider rapidly.

### wscat Example

```bash
npx wscat -c ws://localhost:8080/api/v1/ws
# then paste (replace <ID> with a real preset ID):
{"type":"start_simulation","scenario_id":"<ID>"}
```

## Data Models

### SimulationOutput

Returned by `POST /scenarios/:id/run`:

```json
{
  "scenario_id": "uuid",
  "scenario_name": "Business as Usual",
  "timeline": [1900, 1901, ..., 2100],
  "states": [
    {
      "year": 1900,
      "population": { "total": 1.6e9, ... },
      "capital": { "industrial_capital": ..., ... },
      "agriculture": { "arable_land": ..., ... },
      "resources": { "nnr_fraction": ... },
      "pollution": { "persistent_pollution": ..., ... }
    }
  ],
  "params": { ... },
  "computed_at": "2026-02-24T12:00:00Z"
}
```

### ScenarioSummary

Returned by `GET /scenarios` and `GET /presets`:

```json
{
  "id": "uuid",
  "name": "Business as Usual",
  "description": "Original World 3 standard run.",
  "color_hex": "#e63946",
  "is_preset": true
}
```
