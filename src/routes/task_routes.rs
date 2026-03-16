use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::task_handler;
use crate::repositories::task_repository::TaskRepository;

pub fn task_routes(repo: Arc<TaskRepository>) -> Router {
    Router::new()
        .route("/tasks", get(task_handler::list_tasks))
        .route("/tasks", post(task_handler::create_task))
        .route(
            "/tasks/{id}",
            get(task_handler::get_task)
                .put(task_handler::update_task)
                .delete(task_handler::delete_task),
        )
        .with_state(repo)
}
