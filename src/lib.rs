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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::user_handler::list_users,
        handlers::user_handler::get_user,
        handlers::user_handler::create_user,
        handlers::user_handler::update_user,
        handlers::user_handler::delete_user,
        handlers::user_handler::list_user_tasks,
        handlers::project_handler::list_projects,
        handlers::project_handler::get_project,
        handlers::project_handler::create_project,
        handlers::project_handler::update_project,
        handlers::project_handler::delete_project,
        handlers::project_handler::list_project_tasks,
        handlers::board_handler::list_boards,
        handlers::board_handler::get_board,
        handlers::board_handler::create_board,
        handlers::board_handler::update_board,
        handlers::board_handler::delete_board,
        handlers::board_handler::list_board_projects,
        handlers::task_handler::list_tasks,
        handlers::task_handler::get_task,
        handlers::task_handler::create_task,
        handlers::task_handler::update_task,
        handlers::task_handler::delete_task,
        handlers::column_handler::list_board_columns,
        handlers::column_handler::get_column,
        handlers::column_handler::create_column,
        handlers::column_handler::update_column,
        handlers::column_handler::delete_column,
        handlers::column_handler::list_column_tasks,
        handlers::column_handler::move_task_to_column,
    ),
    components(schemas(
        models::user::User,
        models::user::CreateUser,
        models::user::UpdateUser,
        models::project::Project,
        models::project::CreateProject,
        models::project::UpdateProject,
        models::board::Board,
        models::board::CreateBoard,
        models::board::UpdateBoard,
        models::task::Task,
        models::task::CreateTask,
        models::task::UpdateTask,
        models::column::Column,
        models::column::CreateColumn,
        models::column::UpdateColumn,
        models::column::MoveTask,
    )),
    tags(
        (name = "Users", description = "User management"),
        (name = "Projects", description = "Project management"),
        (name = "Boards", description = "Board management"),
        (name = "Tasks", description = "Task management"),
        (name = "Columns", description = "Column management"),
    ),
    info(
        title = "Tulsi API",
        version = "0.1.0",
        description = "Tulsi project management API"
    )
)]
struct ApiDoc;

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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_handler).with_state(pool))
        .route("/metrics", get(metrics_handler).with_state(prometheus_handle))
        .layer(TraceLayer::new_for_http())
}
