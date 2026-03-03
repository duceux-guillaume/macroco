#!/usr/bin/env bash
# Quick start: ./run.sh
# Starts the Macroco backend API server and frontend dev server.
# Press Ctrl+C to stop both.

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

# Trap SIGINT/SIGTERM to kill both processes
cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
    wait $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
    echo -e "${GREEN}Done.${NC}"
    exit 0
}
trap cleanup SIGINT SIGTERM

# Start backend
echo -e "${YELLOW}Starting backend (first build may take a few minutes)...${NC}"
(cd "$PROJECT_DIR" && RUST_LOG=info cargo run --bin world3-api) &
BACKEND_PID=$!

# Wait briefly for backend to start compiling, then start frontend
sleep 2

echo -e "${YELLOW}Starting frontend...${NC}"
(cd "$PROJECT_DIR/frontend" && npm run dev) &
FRONTEND_PID=$!

echo ""
echo -e "${GREEN}✓ Backend starting on port 8080${NC}"
echo -e "${GREEN}✓ Frontend starting — open http://localhost:5173${NC}"
echo ""
echo "Press Ctrl+C to stop both servers."

# Wait for either process to exit
wait -n $BACKEND_PID $FRONTEND_PID 2>/dev/null || true
cleanup
