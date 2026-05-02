# ─────────────────────────────────────────────────────────────────────────────
# ZenMonitor Server - Multi-stage Dockerfile
# Copyright (c) 2024 ZenLabsAI
# ─────────────────────────────────────────────────────────────────────────────

# ── Build Stage ───────────────────────────────────────────────────────────────
FROM rust:1.77-bookworm AS builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY server/ server/
COPY agent/ agent/

# Build both packages in release mode
RUN cargo build --release --package zenmonitor-server && \
    cargo build --release --package zenmonitor-agent

# Copy dashboard
COPY dashboard/ dashboard/

# ── Server Runtime Stage ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        iputils-ping \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/zenmonitor-server /app/zenmonitor-server
COPY --from=builder /build/target/release/zenmonitor-agent /app/zenmonitor-agent

# Copy dashboard static files
COPY --from=builder /build/dashboard/static /app/dashboard/static

# Create data directory
RUN mkdir -p /data

# Default config
COPY zenmonitor-server.toml /app/zenmonitor-server.toml

EXPOSE 3000

ENV ZENMONITOR_CONFIG=/app/zenmonitor-server.toml
ENV RUST_LOG=zenmonitor_server=info

ENTRYPOINT ["/app/zenmonitor-server"]
