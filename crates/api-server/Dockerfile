FROM rust:1.93 as builder
WORKDIR /usr/src/orchestrator-rs

# Copy the whole repo for workspace build
COPY . .

WORKDIR /usr/src/orchestrator-rs
# Build the workspace and the api-server binary in release mode
RUN cargo build --release --workspace -p api-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/orchestrator-rs/target/release/api-server /usr/local/bin/api-server
WORKDIR /app
COPY config.toml.example /app/config.toml.example
EXPOSE 8080
CMD ["/usr/local/bin/api-server"]
