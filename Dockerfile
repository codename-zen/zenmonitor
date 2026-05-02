# ─────────────────────────────────────────────────────────────────────────────
# ZenMonitor Server - Multi-stage Dockerfile
# Copyright (c) 2024 ZenLabsAI
# ─────────────────────────────────────────────────────────────────────────────

# ── Build Stage ───────────────────────────────────────────────────────────────
FROM rust:1.85-bookworm AS builder

WORKDIR /build

# Limit parallel jobs to avoid OOM on low-memory servers
ENV CARGO_BUILD_JOBS=2
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src files for dependency pre-build cache
RUN mkdir -p server/src agent/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > agent/src/main.rs && \
    cargo build --release --package zenmonitor-server 2>/dev/null || true && \
    cargo build --release --package zenmonitor-agent 2>/dev/null || true && \
    rm -rf server/src agent/src

# Copy actual source
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
