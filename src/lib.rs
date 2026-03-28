pub mod handlers;
pub mod models;
pub mod observability;
pub mod repositories;
pub mod routes;

use std::sync::Arc;

use axum::{Router, routing::get};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use observability::{health_handler, metrics_handler};
use repositories::board_repository::BoardRepository;
use repositories::column_repository::ColumnRepository;
use repositories::project_repository::ProjectRepository;
use repositories::task_repository::TaskRepository;
use repositories::user_repository::UserRepository;
use routes::board_routes::board_routes;
use routes::column_routes::column_routes;
use routes::project_routes::project_routes;
use routes::task_routes::task_routes;
use routes::user_routes::user_routes;

pub fn build_app(pool: PgPool, prometheus_handle: PrometheusHandle) -> Router {
    let task_repo = Arc::new(TaskRepository::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepository::new(pool.clone()));
    let board_repo = Arc::new(BoardRepository::new(pool.clone()));
    let column_repo = Arc::new(ColumnRepository::new(pool.clone()));
    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    task_routes(task_repo)
        .merge(project_routes(project_repo))
        .merge(board_routes(board_repo))
        .merge(column_routes(column_repo))
        .merge(user_routes(user_repo))
        .route("/health", get(health_handler).with_state(pool))
        .route("/metrics", get(metrics_handler).with_state(prometheus_handle))
        .layer(TraceLayer::new_for_http())
}
