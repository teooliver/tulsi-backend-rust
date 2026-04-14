pub mod auth;
pub mod cache;
pub mod handlers;
pub mod models;
pub mod observability;
pub mod repositories;
pub mod routes;

use std::sync::Arc;

use axum::{Extension, Router, middleware, routing::get};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use auth::{JwtSecret, require_auth};
use cache::RedisCache;
use observability::{health_handler, metrics_handler};
use repositories::board_repository::BoardRepository;
use repositories::column_repository::ColumnRepository;
use repositories::project_repository::ProjectRepository;
use repositories::task_history_repository::TaskHistoryRepository;
use repositories::task_repository::TaskRepository;
use repositories::user_repository::UserRepository;
use routes::auth_routes::auth_routes;
use routes::board_routes::board_routes;
use routes::column_routes::column_routes;
use routes::project_routes::project_routes;
use routes::task_routes::task_routes;
use routes::user_routes::user_routes;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth_handler::register,
        handlers::auth_handler::login,
        handlers::auth_handler::me,
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
        handlers::task_history_handler::get_task_history,
    ),
    components(schemas(
        models::auth::RegisterRequest,
        models::auth::LoginRequest,
        models::auth::AuthResponse,
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
        models::task_history::TaskHistory,
        models::task_history::TaskEventType,
    )),
    tags(
        (name = "Auth", description = "Authentication"),
        (name = "Users", description = "User management"),
        (name = "Projects", description = "Project management"),
        (name = "Boards", description = "Board management"),
        (name = "Tasks", description = "Task management"),
        (name = "Columns", description = "Column management"),
        (name = "Task History", description = "Task change history"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Tulsi API",
        version = "0.1.0",
        description = "Tulsi project management API"
    )
)]
struct ApiDoc;

pub fn build_app(pool: PgPool, cache: Option<RedisCache>, prometheus_handle: PrometheusHandle) -> Router {
    let jwt_secret = JwtSecret(
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-change-me".to_string())
            .into(),
    );

    let task_repo = Arc::new(TaskRepository::new(pool.clone(), cache.clone()));
    let task_history_repo = Arc::new(TaskHistoryRepository::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepository::new(pool.clone(), cache.clone()));
    let board_repo = Arc::new(BoardRepository::new(pool.clone(), cache.clone()));
    let column_repo = Arc::new(ColumnRepository::new(pool.clone(), cache.clone()));
    let user_repo = Arc::new(UserRepository::new(pool.clone(), cache));

    let public_routes = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_handler).with_state(pool))
        .route("/metrics", get(metrics_handler).with_state(prometheus_handle))
        .merge(auth_routes(user_repo.clone()));

    let protected_routes = Router::new()
        .merge(task_routes(task_repo))
        .merge(project_routes(project_repo))
        .merge(board_routes(board_repo))
        .merge(column_routes(column_repo))
        .merge(user_routes(user_repo))
        .layer(middleware::from_fn(require_auth))
        .layer(Extension(task_history_repo));

    let all_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(Extension(jwt_secret))
        .layer(TraceLayer::new_for_http());

    all_routes
}
