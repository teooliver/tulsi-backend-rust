use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
};
use sqlx::PgPool;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Initialize the full observability stack: tracing, metrics, and optionally OTLP export.
///
/// Returns a `PrometheusHandle` to serve the `/metrics` endpoint.
pub fn init_observability() -> PrometheusHandle {
    // Prometheus metrics recorder
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    // Env filter: defaults to info, configurable via RUST_LOG
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    // JSON or pretty format based on env
    let json_logging = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    // Try to set up OTLP if endpoint is configured
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otlp_endpoint {
        let tracer_provider = build_tracer_provider(&endpoint);
        let tracer = tracer_provider.tracer("tulsi-rust-backend");
        build_meter_provider(&endpoint);

        if json_logging {
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            Registry::default()
                .with(env_filter)
                .with(fmt::layer().json())
                .with(otel_layer)
                .init();
        } else {
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            Registry::default()
                .with(env_filter)
                .with(fmt::layer())
                .with(otel_layer)
                .init();
        }

        opentelemetry::global::set_tracer_provider(tracer_provider);
        tracing::info!(otlp.endpoint = %endpoint, "OpenTelemetry OTLP export enabled");
    } else {
        if json_logging {
            Registry::default()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        } else {
            Registry::default()
                .with(env_filter)
                .with(fmt::layer())
                .init();
        }

        tracing::info!("OpenTelemetry OTLP export disabled (set OTEL_EXPORTER_OTLP_ENDPOINT to enable)");
    }

    prometheus_handle
}

fn build_tracer_provider(endpoint: &str) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("failed to create OTLP trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build()
}

fn build_meter_provider(endpoint: &str) {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("failed to create OTLP metrics exporter");

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build();

    opentelemetry::global::set_meter_provider(meter_provider);
}

/// Handler for GET /metrics — returns Prometheus-format metrics.
pub async fn metrics_handler(
    State(handle): State<PrometheusHandle>,
) -> impl IntoResponse {
    handle.render()
}

/// Handler for GET /health — checks database connectivity.
pub async fn health_handler(
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "database unreachable"),
    }
}
