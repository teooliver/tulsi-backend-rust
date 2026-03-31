# Stage 1: Build
FROM rust:1.88 AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/tulsi-seed/Cargo.toml crates/tulsi-seed/Cargo.toml

# Create dummy source files so cargo can resolve the workspace and cache deps
RUN mkdir -p src crates/tulsi-seed/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > crates/tulsi-seed/src/main.rs

RUN cargo build --release && rm -rf src crates/tulsi-seed/src

# Copy real source code
COPY src src
COPY crates crates
COPY migrations migrations

# Touch so cargo detects the change
RUN touch src/main.rs src/lib.rs crates/tulsi-seed/src/main.rs

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tulsi-rust-backend /usr/local/bin/tulsi-rust-backend

# Removing this for now, the value will be injected at runtime by K3s.
# ENV DATABASE_URL=postgres://db_user_test:12345@db:5432/tulsi_test_db

EXPOSE 3000

CMD ["tulsi-rust-backend"]
