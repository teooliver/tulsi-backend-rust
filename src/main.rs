use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

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
    sqlx::raw_sql(include_str!("../migrations/004_create_users.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");
    sqlx::raw_sql(include_str!("../migrations/005_create_columns.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    let app = tulsi_rust_backend::build_app(pool).layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
