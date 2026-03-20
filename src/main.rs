use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

mod handlers;
mod models;
mod repositories;
mod routes;

use repositories::board_repository::BoardRepository;
use repositories::project_repository::ProjectRepository;
use repositories::task_repository::TaskRepository;
use routes::board_routes::board_routes;
use routes::project_routes::project_routes;
use routes::task_routes::task_routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://db_user_test:12345@localhost:5432/tulsi_test_db".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::raw_sql(include_str!("../migrations/001_create_tasks.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");
    sqlx::raw_sql(include_str!("../migrations/002_create_projects.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");
    sqlx::raw_sql(include_str!("../migrations/003_create_boards.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    let task_repo = Arc::new(TaskRepository::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepository::new(pool.clone()));
    let board_repo = Arc::new(BoardRepository::new(pool));

    let app = task_routes(task_repo)
        .merge(project_routes(project_repo))
        .merge(board_routes(board_repo))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
