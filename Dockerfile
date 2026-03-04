# --- Build stage ---
FROM rust:latest AS builder

WORKDIR /app

# Copy workspace manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/cli/Cargo.toml crates/cli/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# Create stub source files so cargo can resolve deps
RUN mkdir -p crates/core/src crates/cli/src crates/server/src && \
    echo "fn main() {}" > crates/cli/src/main.rs && \
    echo "fn main() {}" > crates/server/src/main.rs && \
    echo "" > crates/core/src/lib.rs

# Pre-build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release -p agentbin-server 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/
COPY static/ static/

# Touch sources so cargo rebuilds them (not the cached stubs)
RUN touch crates/core/src/lib.rs crates/cli/src/main.rs crates/server/src/main.rs

# Build the server binary
RUN cargo build --release -p agentbin-server

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/agentbin-server /usr/local/bin/agentbin-server

# Create a non-root user
RUN useradd --create-home --shell /bin/bash agentbin
USER agentbin

# Default storage path
ENV AGENTBIN_STORAGE_PATH=/data
ENV AGENTBIN_LOG_FORMAT=json
ENV AGENTBIN_LISTEN_ADDR=0.0.0.0:8080

EXPOSE 8080

CMD ["agentbin-server"]
