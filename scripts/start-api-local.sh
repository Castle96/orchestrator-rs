#!/usr/bin/env bash
set -euo pipefail

# Helper to run the API locally with options for development.
# Supports: --rebuild, --config <path>, --secret <jwt>, --detach, --help

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

usage() {
	cat <<EOF
Usage: $(basename "$0") [--rebuild] [--config <path>] [--secret <jwt>] [--detach]

Options:
	--rebuild          Run 'cargo build --workspace' before launching
	--config <path>    Path to config file (default: ./config.toml)
	--secret <jwt>     JWT secret to export as JWT_SECRET (overrides env)
	--detach           Run server in background (logs to ./logs/api.log)
	-h, --help         Show this help
EOF
}

REBUILD=false
DETACH=false
CONFIG_PATH="$ROOT_DIR/config.toml"
SECRET=""

while [[ $# -gt 0 ]]; do
	case "$1" in
		--rebuild) REBUILD=true; shift ;;
		--detach) DETACH=true; shift ;;
		--config) CONFIG_PATH="$2"; shift 2 ;;
		--secret) SECRET="$2"; shift 2 ;;
		-h|--help) usage; exit 0 ;;
		*) echo "Unknown arg: $1"; usage; exit 1 ;;
	esac
done

if ! command -v cargo >/dev/null 2>&1; then
	echo "cargo not found. Install Rust toolchain (rustup) before using this script." >&2
	exit 1
fi

cd "$ROOT_DIR"

if $REBUILD; then
	echo "Rebuilding workspace..."
	cargo build --workspace
fi

# Determine JWT secret: CLI override > env
if [[ -n "$SECRET" ]]; then
	export JWT_SECRET="$SECRET"
elif [[ -z "${JWT_SECRET:-}" ]]; then
	echo "Warning: JWT_SECRET not set; authentication may fail." >&2
fi

if [[ ! -f "$CONFIG_PATH" ]]; then
	echo "Warning: config file '$CONFIG_PATH' not found; server will use defaults if available." >&2
fi

if $DETACH; then
	mkdir -p "$ROOT_DIR/logs"
	echo "Starting api-server in background, logs: $ROOT_DIR/logs/api.log"
	nohup env JWT_SECRET="$JWT_SECRET" cargo run -p api-server > "$ROOT_DIR/logs/api.log" 2>&1 &
	echo $! > "$ROOT_DIR/logs/api.pid"
	echo "api-server PID: $(cat "$ROOT_DIR/logs/api.pid")"
	exit 0
else
	echo "Starting api-server in foreground (CTRL+C to stop)"
	env JWT_SECRET="$JWT_SECRET" cargo run -p api-server
fi

