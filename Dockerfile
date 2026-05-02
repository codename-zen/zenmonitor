# ─────────────────────────────────────────────────────────────────────────────
# ZenMonitor Server - Multi-stage Dockerfile
# Copyright (c) 2024 ZenLabsAI
# ─────────────────────────────────────────────────────────────────────────────

# ── Build Stage ───────────────────────────────────────────────────────────────
FROM rust:1.77-bookworm AS builder

WORKDIR /build

# Cache dependencies by building a dummy project first
COPY Cargo.toml Cargo.lock* ./
COPY server/Cargo.toml server/Cargo.toml
COPY agent/Cargo.toml agent/Cargo.toml

# Create dummy source files to cache dependency compilation
RUN mkdir -p server/src agent/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > agent/src/main.rs && \
    cargo build --release --package zenmonitor-server 2>/dev/null || true && \
    rm -rf server/src agent/src

# Copy actual source code
COPY server/ server/
COPY agent/ agent/

# Build the server binary
RUN cargo build --release --package zenmonitor-server

# ── Server Runtime Stage ──────────────────────────────────────────────────────
FROM debian:bookworm-slim AS server

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        iputils-ping \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --system --no-create-home --shell /usr/sbin/nologin zenmonitor

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/zenmonitor-server /app/zenmonitor-server

# Copy dashboard static files if they exist
COPY --from=builder /build/server/dashboard/ /app/dashboard/ 2>/dev/null || true

# Create data directory
RUN mkdir -p /data && chown zenmonitor:zenmonitor /data

# Default config
COPY zenmonitor-server.toml /app/zenmonitor-server.toml

USER zenmonitor

EXPOSE 3000

ENV ZENMONITOR_CONFIG=/app/zenmonitor-server.toml
ENV RUST_LOG=zenmonitor_server=info

ENTRYPOINT ["/app/zenmonitor-server"]
