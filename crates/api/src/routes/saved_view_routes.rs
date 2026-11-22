use std::sync::Arc;

use axum::{
    Router,
    routing::get,
};

use crate::handlers::saved_view_handler;
use repositories::saved_view_repository::SavedViewRepository;

pub fn saved_view_routes(repo: Arc<SavedViewRepository>) -> Router {
    Router::new()
        .route(
            "/saved-views",
            get(saved_view_handler::list_saved_views).post(saved_view_handler::create_saved_view),
        )
        .route(
            "/saved-views/{id}",
            get(saved_view_handler::get_saved_view)
                .put(saved_view_handler::update_saved_view)
                .delete(saved_view_handler::delete_saved_view),
        )
        .route("/saved-views/{id}/tasks", get(saved_view_handler::execute_saved_view))
        .with_state(repo)
}
