# Stage 1: Build
# Keep this Rust version in sync with .devcontainer/post-create.sh (RUST_VERSION).
FROM rust:1.88.0 AS builder

WORKDIR /app

# Copy workspace + per-crate manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/models/Cargo.toml crates/models/Cargo.toml
COPY crates/auth/Cargo.toml crates/auth/Cargo.toml
COPY crates/cache/Cargo.toml crates/cache/Cargo.toml
COPY crates/observability/Cargo.toml crates/observability/Cargo.toml
COPY crates/repositories/Cargo.toml crates/repositories/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml
COPY crates/seed/Cargo.toml crates/seed/Cargo.toml

# Create dummy source files so cargo can resolve the workspace and cache deps
RUN for c in models auth cache observability repositories api; do \
        mkdir -p crates/$c/src && echo "" > crates/$c/src/lib.rs; \
    done \
    && mkdir -p crates/server/src crates/seed/src \
    && echo "fn main() {}" > crates/server/src/main.rs \
    && echo "fn main() {}" > crates/seed/src/main.rs

RUN cargo build --release && rm -rf crates/*/src

# Copy real source code
COPY crates crates
COPY migrations migrations

RUN cargo build --release --bin server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/server /usr/local/bin/server

# Removing this for now, the value will be injected at runtime by K3s.
# ENV DATABASE_URL=postgres://db_user_test:12345@db:5432/tulsi_test_db

EXPOSE 8080

CMD ["server"]
