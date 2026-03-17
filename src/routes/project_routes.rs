use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::project_handler;
use crate::repositories::project_repository::ProjectRepository;

pub fn project_routes(repo: Arc<ProjectRepository>) -> Router {
    Router::new()
        .route("/projects", get(project_handler::list_projects))
        .route("/projects", post(project_handler::create_project))
        .route(
            "/projects/{id}",
            get(project_handler::get_project)
                .put(project_handler::update_project)
                .delete(project_handler::delete_project),
        )
        .route(
            "/projects/{id}/tasks",
            get(project_handler::list_project_tasks),
        )
        .with_state(repo)
}
