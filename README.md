# Macroco

Online live macroeconomic model based on the World 3 system dynamics model
(Meadows et al., *Limits to Growth*). Extended with modern indicators: climate,
energy mix, biodiversity, inequality.

## Quick Start

> See the full [Quick Start Guide](docs/quick-start.md) for platform-specific instructions.

```bash
git clone <repo-url> && cd macroco
./run.sh            # builds + serves on http://localhost:5173
```

Requires Rust 1.75+ and Node.js 18+.

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
| [Chart Output](docs/chart-output.md) | PNG chart rendering |
| [Deployment](docs/deployment.md) | Fly.io deployment guide |
| [Lookup Table Audit](docs/audit.md) | Audit against pyworld3 reference |

## License

GPL v3 — see [LICENSE](LICENSE).
