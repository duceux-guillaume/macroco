# ── Stage 1: Build Rust binary ────────────────────────────────────────────────
FROM rust:1.85-bookworm AS rust-builder

WORKDIR /app

# Install system deps for plotters (fontconfig)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/world3-core/Cargo.toml crates/world3-core/Cargo.toml
COPY crates/world3-api/Cargo.toml crates/world3-api/Cargo.toml
COPY crates/world3-ingestion/Cargo.toml crates/world3-ingestion/Cargo.toml
COPY crates/world3-cli/Cargo.toml crates/world3-cli/Cargo.toml

# Create stub source files so cargo can resolve the workspace and cache deps
RUN mkdir -p crates/world3-core/src && echo "pub fn stub(){}" > crates/world3-core/src/lib.rs \
    && mkdir -p crates/world3-api/src && echo "fn main(){}" > crates/world3-api/src/main.rs \
    && mkdir -p crates/world3-ingestion/src && echo "pub fn stub(){}" > crates/world3-ingestion/src/lib.rs \
    && mkdir -p crates/world3-cli/src && echo "fn main(){}" > crates/world3-cli/src/main.rs

# Stubs won't compile fully — errors are expected. This step only caches
# downloaded and partially-compiled dependencies so the real build is faster.
RUN cargo build --release --bin world3-api 2>/dev/null || true

# Copy real source and rebuild
COPY crates/ crates/
RUN touch crates/world3-core/src/lib.rs \
    crates/world3-api/src/main.rs \
    crates/world3-ingestion/src/lib.rs \
    crates/world3-cli/src/main.rs \
    && cargo build --release --bin world3-api

# ── Stage 2: Build frontend ──────────────────────────────────────────────────
FROM node:22-bookworm-slim AS frontend-builder

WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ .
RUN npm run build

# ── Stage 3: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --no-create-home --shell /sbin/nologin appuser

WORKDIR /app

# Binary
COPY --from=rust-builder /app/target/release/world3-api .

# Static frontend
COPY --from=frontend-builder /app/frontend/build ./static

# Data files (presets, lookup tables) and license notices
COPY data/ ./data/
COPY LICENSE THIRD_PARTY_LICENSES ./

RUN chown -R appuser:appuser /app
USER appuser

ENV PORT=8080
ENV STATIC_DIR=/app/static

EXPOSE 8080

CMD ["./world3-api"]
