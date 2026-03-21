use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::user_handler;
use crate::repositories::user_repository::UserRepository;

pub fn user_routes(repo: Arc<UserRepository>) -> Router {
    Router::new()
        .route("/users", get(user_handler::list_users))
        .route("/users", post(user_handler::create_user))
        .route(
            "/users/{id}",
            get(user_handler::get_user)
                .put(user_handler::update_user)
                .delete(user_handler::delete_user),
        )
        .route("/users/{id}/tasks", get(user_handler::list_user_tasks))
        .with_state(repo)
}
