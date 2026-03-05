# Deployment

Macroco deploys as a single container on [Fly.io](https://fly.io). The `world3-api` Rust binary serves both the REST/WebSocket API and the SvelteKit static frontend.

## Architecture

```
┌─────────────────────────────────────────┐
│           Fly.io Machine (cdg)          │
│                                         │
│  world3-api binary                      │
│  ├── /api/v1/*    → Axum REST + WS     │
│  └── /*           → Static files (SPA)  │
│                                         │
│  Estimated cost: ~$5/mo (auto-stop)     │
└─────────────────────────────────────────┘
```

- Single `shared-cpu-1x` machine with 512MB RAM
- Auto-stops after idle period, auto-starts on incoming request
- HTTPS enforced via Fly.io edge proxy
- WebSocket connections work over `wss://` automatically

## Prerequisites

Install the Fly.io CLI:

```bash
curl -L https://fly.io/install.sh | sh
```

Authenticate:

```bash
flyctl auth login
```

## First-Time Setup

1. Create the app:

```bash
flyctl apps create macroco
```

2. Deploy:

```bash
flyctl deploy --remote-only
```

3. Open in browser:

```bash
flyctl open
```

The app will be available at `https://macroco.fly.dev`.

## CI/CD (Automated Deploy)

Pushes to `main` that pass clippy and tests are automatically deployed via GitHub Actions.

### Setup

1. Create a deploy token:

```bash
flyctl tokens create deploy -x 999999h
```

2. Add the token to GitHub:
   - Go to your repo → Settings → Secrets and variables → Actions
   - Create a secret named `FLY_API_TOKEN` with the token value

### Pipeline

```
Push to main → Clippy → Tests → Deploy to Fly.io
```

### PR Preview Deploy

Pull requests can be deployed for live testing by adding the `deploy-preview` label:

1. Open a PR and wait for CI tests to pass
2. Add the `deploy-preview` label to the PR
3. CI deploys the PR branch to https://macroco.fly.dev
4. Test on the live URL
5. Merge, close, or remove the label — CI auto-reverts to `main`

Only one PR can be previewed at a time. The production URL serves the PR code during preview.

## Manual Deploy

```bash
flyctl deploy --remote-only
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | TCP port (Fly.io sets this automatically) |
| `RUST_LOG` | `info` | Log level filter |
| `STATIC_DIR` | `/app/static` | Path to frontend static files |

These are configured in `fly.toml` and baked into the Docker image. To override at runtime:

```bash
flyctl secrets set RUST_LOG=debug
```

## Docker

The `Dockerfile` uses a multi-stage build:

1. **Rust builder** (`rust:1.85-bookworm`): Compiles the `world3-api` release binary with dependency caching
2. **Frontend builder** (`node:22-bookworm-slim`): Runs `npm ci && npm run build` to produce static files
3. **Runtime** (`debian:bookworm-slim`): Copies binary + static files into a ~100MB image

### Build locally

```bash
docker build -t macroco .
docker run -p 8080:8080 macroco
```

Visit `http://localhost:8080`.

## Health Check

The server exposes `GET /api/v1/health` which returns:

```json
{"status": "ok", "version": "0.1.0"}
```

Fly.io checks this every 15 seconds with a 30-second grace period on cold start.

## Monitoring

View logs:

```bash
flyctl logs
```

Check machine status:

```bash
flyctl status
```

SSH into the running machine:

```bash
flyctl ssh console
```

## Graceful Shutdown

The server handles `SIGTERM` (sent by Fly.io before stopping) and drains active connections before exiting. WebSocket clients will automatically reconnect when the machine restarts.
