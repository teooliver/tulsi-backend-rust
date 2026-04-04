use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tulsi_rust_backend::cache::RedisCache;
use tulsi_rust_backend::observability;

#[tokio::main]
async fn main() {
    let prometheus_handle = observability::init_observability();

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

    // Connect to Redis (optional — app works without it)
    let redis_cache = match std::env::var("REDIS_URL") {
        Ok(redis_url) => {
            match redis::Client::open(redis_url.as_str()) {
                Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                    Ok(conn) => {
                        tracing::info!("Connected to Redis at {redis_url}");
                        Some(RedisCache::new(conn))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to Redis: {e}. Running without cache.");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!("Invalid REDIS_URL: {e}. Running without cache.");
                    None
                }
            }
        }
        Err(_) => {
            tracing::info!("REDIS_URL not set. Running without cache.");
            None
        }
    };

    let app = tulsi_rust_backend::build_app(pool, redis_cache, prometheus_handle)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
