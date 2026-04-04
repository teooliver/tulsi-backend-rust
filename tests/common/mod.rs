use axum::Router;
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

// Used by http_*_test.rs and workflow_kanban_test.rs
// This is a common pattern for test helper modules — Rust's dead code analysis doesn't see cross-file usage within the `tests/` directory.
#[allow(dead_code)]
pub async fn setup_test_app() -> Router {
    let pool = setup_test_db().await;
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");
    tulsi_rust_backend::build_app(pool, None, prometheus_handle)
}

pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://db_user_test:12345@localhost:5433/tulsi_test_db_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations (all use IF NOT EXISTS except tasks table)
    sqlx::raw_sql(
        "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";
         CREATE TABLE IF NOT EXISTS tasks (
             id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
             title VARCHAR(255) NOT NULL,
             description TEXT NOT NULL DEFAULT '',
             created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
             updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
         );",
    )
    .execute(&pool)
    .await
    .expect("Failed to run migration 001");

    sqlx::raw_sql(include_str!("../../migrations/002_create_projects.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 002");
    sqlx::raw_sql(include_str!("../../migrations/003_create_boards.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 003");
    sqlx::raw_sql(include_str!("../../migrations/004_create_users.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 004");
    sqlx::raw_sql(include_str!("../../migrations/005_create_columns.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 005");

    // Clean data before each test
    sqlx::raw_sql("TRUNCATE tasks, columns, projects, boards, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to clean up test database");

    pool
}
