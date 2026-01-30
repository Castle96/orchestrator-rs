# Development Guide

Quick workflow:

- Ensure Rust toolchain installed (rustup stable) and Node.js 18+.
- Copy `.env.dev.example` -> `.env.dev` and set `JWT_SECRET` (>=32 chars).

Build & test:

```bash
# Build and test Rust workspace
cargo build --workspace
cargo test --workspace

# Build web UI
cd crates/web-ui
npm ci
npm run build
```

Run dev compose (dev profile):

```bash
cd /home/kyle/projects/rust/orchestrator-rs
docker compose --profile dev -f docker-compose.yml -f docker-compose.dev.yml up --build -d
```

Integration tests:

```bash
# Option A: let the test start/stop compose
export RUN_COMPOSE_TESTS=1
cargo test -p api-server --test compose_integration -- --nocapture

# Option B: start compose yourself and run test against running stack
docker compose --profile dev -f docker-compose.yml -f docker-compose.dev.yml up --build -d
CI= RUN_COMPOSE_TESTS=0 cargo test -p api-server --test compose_integration -- --nocapture
```

Pre-commit:

```bash
pip install pre-commit
pre-commit install
pre-commit run --all-files
```

Notes:
- Do not commit `.env.dev` or any secrets. Use `.env.dev.example` as a template.
- For integration tests that compile native TLS crates, ensure system deps are installed:
  - Debian/Ubuntu: `sudo apt install pkg-config libssl-dev`
