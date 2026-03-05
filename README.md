# Macroco

What if you could simulate the future of humanity on your laptop? [Macroco](https://macroco.fly.dev) is a living model of the world — population, resources, food, pollution — calibrated against 60 years of real data. Choose what you believe: that we overshoot and collapse, that technology saves us, or that we learn to live within our means. Then watch what the math says happens next.

[Explore the model →](docs/model/README.md)

## What you can do

- **Pick a scenario** — Collapse, Technotopia, or Ecotopia — and run it from 1900 to 2100
- **Compare futures** side by side with real historical data overlaid
- **Tune parameters** — resource discovery, technology growth, pollution controls — and see what shifts
- **Dig into the model** — every variable, equation, and feedback loop is documented

## Roadmap

| | Milestone | Status |
|---|-----------|--------|
| ●●●●● | **Foundation** — Engine, interactive frontend, API, CLI, docs | Complete |
| ●●●●○ | **Collapse** — Historical calibration against real-world data (1960–2023) | In progress |
| ○○○○○ | **Technotopia** — Climate & energy sectors, technology scenario | Planned |
| ○○○○○ | **Ecotopia** — Biodiversity & inequality sectors, ecotopia scenario | Planned |
| ○○○○○ | **Living Data** — Auto-updating data from World Bank, NOAA, FAO, UN, BP | Planned |
| ○○○○○ | **Deep Exploration** — Sensitivity analysis, advanced charting | Planned |

## Documentation

| Document | Description |
|----------|-------------|
| [Quick Start](docs/quick-start.md) | Install prerequisites and run locally |
| [Model Documentation](docs/model/README.md) | Sectors, equations, lookup tables, deviations from World3-03 |
| [Product Requirements](docs/product-requirements.md) | Feature requirements with stable REQ-NNN IDs |
| [Architecture](docs/architecture.md) | System design, components, data flow |
| [Simulation Engine](docs/simulation-engine.md) | ODE model, sectors, RK4 solver |
| [CLI Reference](docs/cli.md) | Batch simulation commands and flags |
| [API Server](docs/api-server.md) | REST endpoints, WebSocket streaming |
| [Deployment](docs/deployment.md) | Fly.io deployment guide |

## Contributing

See the [Quick Start guide](docs/quick-start.md) to set up a local development environment. Requirements are tracked in [Product Requirements](docs/product-requirements.md) with stable REQ-NNN IDs.
