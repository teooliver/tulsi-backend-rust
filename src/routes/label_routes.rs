use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::label_handler;
use crate::repositories::label_repository::LabelRepository;

pub fn label_routes(repo: Arc<LabelRepository>) -> Router {
    Router::new()
        .route(
            "/labels",
            get(label_handler::list_labels).post(label_handler::create_label),
        )
        .route(
            "/labels/{id}",
            get(label_handler::get_label)
                .put(label_handler::update_label)
                .delete(label_handler::delete_label),
        )
        .route("/tasks/{task_id}/labels", get(label_handler::list_task_labels))
        .route(
            "/tasks/{task_id}/labels/{label_id}",
            post(label_handler::attach_label_to_task)
                .delete(label_handler::detach_label_from_task),
        )
        .with_state(repo)
}
