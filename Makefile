SHELL := /bin/bash

.PHONY: help build test up up-dev api-dev web-dev down logs run-local

help:
	@echo "Available targets:"
	@echo "  make build       # Build the Rust workspace"
	@echo "  make test        # Run workspace tests"
	@echo "  make up          # Start docker compose (build images)"
	@echo "  make up-dev      # Start docker compose in dev mode (bind mounts)"
	@echo "  make api-dev     # Start only API service via docker compose"
	@echo "  make web-dev     # Start only web service via docker compose"
	@echo "  make down        # Stop docker compose"
	@echo "  make logs        # Follow docker compose logs"
	@echo "  make run-local   # Run API locally (cargo run) with JWT_SECRET set"

build:
	cargo build --workspace

test:
	cargo test --workspace

up:
	docker compose up --build

up-dev:
	docker compose up --build

api-dev:
	docker compose up api

web-dev:
	docker compose up web

down:
	docker compose down

logs:
	docker compose logs -f

run-local:
	@echo "Starting API locally with JWT_SECRET (development). Use CTRL+C to stop."
	JWT_SECRET="dev-secret-32-characters-xxxxxxxx" cargo run -p api-server
