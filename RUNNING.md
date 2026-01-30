# Running the project (development)

Prerequisites:
- Rust toolchain (stable) installed via `rustup`.
- Node.js / npm (for the web UI).

Quick start (Linux):

1. Install Rust (if needed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup update stable
```

2. Build and test the Rust workspace:
```bash
cd $(dirname "$0")
cargo build --workspace
cargo test --workspace
```

3. Run the API server (dev):
- For development, set a JWT secret so authentication starts:
```bash
export JWT_SECRET="replace-with-a-32+ char secret"
cargo run -p api-server
```
The server binds to `0.0.0.0:8080` by default and loads `./config.toml` or `/etc/arm-hypervisor/config.toml` if present.

4. Run the web UI (dev):
```bash
cd crates/web-ui
npm install
npm run dev
```
The Vite dev server runs on `http://localhost:3000` and proxies `/api` to the API server at `http://localhost:8080`.

Stopping:
- Use Ctrl+C in the terminal where the server or Vite is running.

Notes:
- The example config enables authentication by default; for quick local testing, setting `JWT_SECRET` is sufficient.
- TLS is disabled by default; do not use this configuration in production.
- Adjust `config.toml` as needed for persistence paths, CORS origins, and TLS certs.

Docker Compose (quick local integration):

1. Build and start services (API + web UI) with a test JWT secret:
```bash
cd /home/kyle/projects/rust/orchestrator-rs
sudo docker compose up --build -d
```
2. Recommended (safer): use the dev override and a local `.env.dev` file to keep secrets out of the repo.

```bash
cd /home/kyle/projects/rust/orchestrator-rs
# copy the example env and set a secure JWT_SECRET locally
cp .env.dev.example .env.dev
# edit .env.dev and set JWT_SECRET (must be >=32 chars)

# bring up the dev stack (uses docker-compose.dev override which reads .env.dev)
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build -d
```

3. To view logs:
```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f api-dev
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f web
```

4. To stop and remove:
```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml down
```

Notes:
- Use `config.toml.example` as a template; copy it to `./config.toml` or provide values via environment variables (preferred).
- Do NOT commit real secrets. Use `.env.dev` locally and ensure `.env.dev` is listed in `.gitignore`.

System dependencies for integration tests:
- On Debian/Ubuntu install `pkg-config` and OpenSSL dev headers before running the integration test that builds `reqwest` native crates:

```bash
sudo apt update && sudo apt install -y pkg-config libssl-dev
```

Dev compose profile:
- The `api-dev` and `web` development services are exposed under the `dev` profile. To start the dev services with profiles enabled use:

```bash
docker compose --profile dev -f docker-compose.yml -f docker-compose.dev.yml up --build -d
```

Integration test (optional):
- The repository includes an integration test that will bring up the dev compose stack and run smoke checks. To run it locally (it will start/stop compose):

```bash
export RUN_COMPOSE_TESTS=1
cargo test -p api-server --test compose_integration -- --nocapture
```

