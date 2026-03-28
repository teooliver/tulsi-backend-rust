# Observability Setup

## Overview

The observability stack provides structured logging, distributed tracing, and Prometheus metrics for the Tulsi Rust backend.

## New file: `src/observability.rs`

- `init_observability()` — sets up the full stack and returns a `PrometheusHandle`
- Structured logging via `tracing-subscriber` with `EnvFilter` (configurable via `RUST_LOG`)
- JSON log format when `LOG_FORMAT=json`
- OpenTelemetry OTLP export (traces via Tempo, metrics via Mimir) — **only activates** when `OTEL_EXPORTER_OTLP_ENDPOINT` is set
- `GET /metrics` — Prometheus-format metrics endpoint
- `GET /health` — DB connectivity check

## Modified files

- `Cargo.toml` — added `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`, `metrics`, `metrics-exporter-prometheus`, `tower-http` trace feature
- `src/lib.rs` — wired in `TraceLayer` middleware, `/health`, and `/metrics` routes
- `src/main.rs` — replaced `tracing_subscriber::fmt::init()` with `observability::init_observability()`
- `tests/common/mod.rs` — updated for new `build_app` signature

## Environment Variables

| Variable | Purpose | Default |
|---|---|---|
| `RUST_LOG` | Log level filter | `info,sqlx=warn,tower_http=debug` |
| `LOG_FORMAT` | Set to `json` for JSON logs | pretty (human-readable) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Grafana OTLP endpoint (e.g. `http://localhost:4317`) | disabled |

## Connecting to Grafana

When you have a Grafana instance ready, set `OTEL_EXPORTER_OTLP_ENDPOINT` and:
- **Traces** will flow to **Tempo**
- **Metrics** will flow to **Mimir/Prometheus**
- **Logs** can be collected by **Loki** (via a log collector like Grafana Alloy reading the JSON stdout)

The `/metrics` endpoint can also be scraped directly by Prometheus.
