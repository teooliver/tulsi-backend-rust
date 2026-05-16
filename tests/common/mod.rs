use axum::Router;
use axum::http::{HeaderValue, header};
use axum_test::TestServer;
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use uuid::Uuid;

// Used by http_*_test.rs and workflow_kanban_test.rs
// This is a common pattern for test helper modules — Rust's dead code analysis doesn't see cross-file usage within the `tests/` directory.
#[allow(dead_code)]
pub async fn setup_test_app() -> Router {
    let pool = setup_test_db().await;
    // Use a local recorder when the global one is already installed (parallel tests).
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .unwrap_or_else(|_| PrometheusBuilder::new().build_recorder().handle());
    tulsi_rust_backend::build_app(pool, None, prometheus_handle)
}

pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://db_user_test:12345@localhost:5433/tulsi_test_db_test".to_string()
    });

    // Single-connection pool + session-level advisory lock serializes every
    // test across binaries. Lock releases when the pool/connection drops at
    // end of test, so a panicking test still frees the next one.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(60))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::query("SELECT pg_advisory_lock(8723451276)")
        .execute(&pool)
        .await
        .expect("Failed to acquire test database lock");

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
    sqlx::raw_sql(include_str!("../../migrations/006_add_author_to_tasks.sql"))
        .execute(&pool)
        .await
        .ok(); // column may already exist; ignore error
    sqlx::raw_sql(include_str!("../../migrations/007_add_password_hash_to_users.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 007");
    sqlx::raw_sql(include_str!("../../migrations/008_create_task_history.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 008");
    sqlx::raw_sql(include_str!("../../migrations/009_create_plans.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration 009");

    // Clean data before each test (CASCADE handles dependent tables like plans, task_history)
    sqlx::raw_sql(
        "TRUNCATE tasks, columns, projects, boards, users, plans RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await
    .expect("Failed to clean up test database");

    pool
}

/// Registers a new user and returns their JWT token. Each call uses a unique email.
#[allow(dead_code)]
pub async fn auth_token(server: &TestServer) -> String {
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let response = server
        .post("/auth/register")
        .json(&serde_json::json!({
            "name": "Test User",
            "email": email,
            "password": "test_password_123"
        }))
        .await;
    let body: serde_json::Value = response.json();
    body["token"].as_str().unwrap().to_string()
}

/// Mints a JWT for an arbitrary user_id without touching the database.
/// Use this for tests on handlers that don't store the auth user via FK.
#[allow(dead_code)]
pub fn mint_token(user_id: Uuid) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    tulsi_rust_backend::auth::create_token(&secret, user_id, "test@example.com")
        .expect("Failed to mint test JWT")
}

/// Sets up a TestServer with a default Authorization header backed by a real
/// registered user. Use for tests touching handlers that record FK references
/// to the auth user (task_history, plans).
#[allow(dead_code)]
pub async fn authed_server() -> TestServer {
    let mut server = TestServer::new(setup_test_app().await);
    let token = auth_token(&server).await;
    server.add_header(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    server
}

/// Sets up a TestServer with a default Authorization header backed by a minted
/// token (random user_id, no DB row). Use when the handler under test does
/// not need the caller to exist in `users`.
#[allow(dead_code)]
pub async fn fake_authed_server() -> TestServer {
    let mut server = TestServer::new(setup_test_app().await);
    let token = mint_token(Uuid::new_v4());
    server.add_header(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    server
}
