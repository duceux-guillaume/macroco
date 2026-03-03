#!/usr/bin/env bash
# Quick start: ./run.sh
# Builds frontend, then serves everything on http://localhost:8080
#
# Options:
#   --dev   Start Vite dev server with hot-reload (frontend on :5173, backend on :8080)

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check prerequisites
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo is not installed.${NC}"
    echo "Install from https://rustup.rs:"
    echo '  curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh'
    exit 1
fi

if ! command -v node &> /dev/null || ! command -v npm &> /dev/null; then
    echo -e "${RED}Error: Node.js/npm is not installed.${NC}"
    echo "Install Node.js 18+ from https://nodejs.org or via your package manager."
    exit 1
fi

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Install frontend deps if needed
if [ ! -d "$PROJECT_DIR/frontend/node_modules" ]; then
    echo -e "${YELLOW}Installing frontend dependencies...${NC}"
    (cd "$PROJECT_DIR/frontend" && npm install)
fi

# ── Dev mode: Vite hot-reload + backend ──────────────────────────────────────
if [ "$1" = "--dev" ]; then
    cleanup() {
        echo ""
        echo -e "${YELLOW}Shutting down...${NC}"
        kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
        wait $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
        echo -e "${GREEN}Done.${NC}"
        exit 0
    }
    trap cleanup SIGINT SIGTERM

    echo -e "${YELLOW}Starting backend (first build may take a few minutes)...${NC}"
    (cd "$PROJECT_DIR" && RUST_LOG=info cargo run --bin world3-api) &
    BACKEND_PID=$!

    sleep 2

    echo -e "${YELLOW}Starting frontend dev server...${NC}"
    (cd "$PROJECT_DIR/frontend" && npm run dev) &
    FRONTEND_PID=$!

    echo ""
    echo -e "${GREEN}Backend  → http://localhost:8080${NC}"
    echo -e "${GREEN}Frontend → http://localhost:5173${NC}"
    echo ""
    echo "Press Ctrl+C to stop both servers."

    wait -n $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
    cleanup
fi

# ── Default mode: Build frontend, serve everything from backend ──────────────
echo -e "${YELLOW}Building frontend...${NC}"
(cd "$PROJECT_DIR/frontend" && npm run build)

cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    wait $BACKEND_PID 2>/dev/null || true
    echo -e "${GREEN}Done.${NC}"
    exit 0
}
trap cleanup SIGINT SIGTERM

echo -e "${YELLOW}Starting server (first build may take a few minutes)...${NC}"
(cd "$PROJECT_DIR" && RUST_LOG=info STATIC_DIR="$PROJECT_DIR/frontend/build" cargo run --bin world3-api) &
BACKEND_PID=$!

echo ""
echo -e "${GREEN}Macroco starting on http://localhost:8080${NC}"
echo ""
echo "Press Ctrl+C to stop."

wait $BACKEND_PID 2>/dev/null || true
cleanup
