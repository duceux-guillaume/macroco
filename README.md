# Macroco

Explore playfully the possible trajectories of humanity. A living macroeconomic model calibrated against real-world data, built on the World 3 system dynamics model (Meadows et al., *Limits to Growth*). Three belief systems, three futures: Collapse, Technotopia, Ecotopia.

## M1 — Foundation

The platform. Simulation engine, interactive frontend, API, CLI, documentation, CI/CD. Everything needed to run and explore World 3 scenarios.

●●●●●●●●●● 100% (24/24)

| Category | Requirements |
|----------|-------------|
| Engine | REQ-001 Core simulation, REQ-002 Lookup tables, REQ-004 Scenario presets |
| API | REQ-007 REST server, REQ-008 WebSocket streaming |
| UX | REQ-009 SvelteKit app, REQ-020 Variable descriptions, REQ-021 Tooltips, REQ-022 Info panels, REQ-023 Sim controls, REQ-024 Annotations, REQ-025 Preset comparison, REQ-028 Feedback |
| CLI | REQ-003 Batch simulation, REQ-006 Chart output |
| Documentation | REQ-018 Quick start guide, REQ-019 Model guide |
| Infrastructure | REQ-010 CI pipeline, REQ-016 Deployment, REQ-017 Test suite, REQ-027 Traceability |

## M2 — Collapse

The default trajectory: overshoot and decline. BAU scenario polished with historical calibration against real-world data (1960-2023) and validated trajectory matching Meadows 1972 dynamics.

●●●●●●●○○○ 75% (3/4)

| Category | Requirements |
|----------|-------------|
| Validation | REQ-005 Meadows 1972 validation, REQ-026 BAU historical calibration, REQ-031 Trajectory validation |
| Data | REQ-012 Historical data overlay |

## M3 — Technotopia

The belief that discovering resources and technology will allow for a Star Trek future. Technology scenario calibrated with climate and energy extension sectors.

○○○○○○○○○○ 0% (0/2)

| Category | Requirements |
|----------|-------------|
| Engine | REQ-033 Climate and energy sectors |
| Validation | REQ-035 Technology historical calibration |

## M4 — Ecotopia

The belief that humanity will progress toward justice and moderation. Stabilized scenario calibrated with biodiversity and inequality extension sectors.

○○○○○○○○○○ 0% (0/2)

| Category | Requirements |
|----------|-------------|
| Engine | REQ-034 Biodiversity and inequality sectors |
| Validation | REQ-036 Stabilized historical calibration |

## M5 — Living Data

Auto-updating historical data pipeline. Real-time ingestion from World Bank, NOAA, FAO, UN, and BP with SQLite cache and fallback to bundled CSVs.

○○○○○○○○○○ 0% (0/2)

| Category | Requirements |
|----------|-------------|
| Data | REQ-013 Ingestion pipeline, REQ-014 Mapping layer |

## M6 — Deep Exploration

Advanced charting, sensitivity analysis, and performance benchmarks. Tools for deeper investigation of model dynamics.

○○○○○○○○○○ 0% (0/1)

| Category | Requirements |
|----------|-------------|
| Engine | REQ-015 Benchmarks and sensitivity analysis |

## Documentation

| Document | Description |
|----------|-------------|
| [Quick Start](docs/quick-start.md) | Install prerequisites and run locally |
| [Model Guide](docs/model-guide.md) | World 3 sectors, feedback loops, chart interpretation |
| [Product Requirements](docs/product-requirements.md) | Feature requirements with stable REQ-NNN IDs |
| [Architecture](docs/architecture.md) | System design, components, data flow |
| [Simulation Engine](docs/simulation-engine.md) | ODE model, sectors, RK4 solver |
| [CLI Reference](docs/cli.md) | Batch simulation commands and flags |
| [API Server](docs/api-server.md) | REST endpoints, WebSocket streaming |
| [Deployment](docs/deployment.md) | Fly.io deployment guide |
| [Lookup Table Audit](docs/audit.md) | Audit against pyworld3 reference |

## Contributing

See the [Quick Start guide](docs/quick-start.md) to set up a local development environment. Requirements are tracked in [Product Requirements](docs/product-requirements.md) with stable REQ-NNN IDs.

## License

GPL v3 — see [LICENSE](LICENSE).
