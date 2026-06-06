#!/usr/bin/env bash
# Runs once after the dev container is created.
set -euo pipefail

# Named volumes mount as root; hand them to the non-root `vscode` user so cargo
# can write build artifacts and cache crates without sudo.
sudo chown -R vscode:vscode /workspace/target /usr/local/cargo/registry

# Pin the toolchain to match the production Dockerfile (FROM rust:1.88.0).
# This is container-scoped via rustup and does not touch the host's toolchain.
RUST_VERSION=1.88.0
rustup toolchain install "$RUST_VERSION" -c rustfmt -c clippy -c rust-src
rustup default "$RUST_VERSION"

# psql + jq back the debugging / curl flows documented in the README.
sudo apt-get update
sudo apt-get install -y --no-install-recommends postgresql-client jq
sudo rm -rf /var/lib/apt/lists/*

# Pre-download dependencies so the first build is just compilation.
cargo fetch

echo "Dev container ready. Run 'cargo run -p server' and open http://localhost:8080/swagger-ui/"
