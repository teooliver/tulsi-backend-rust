use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use crate::handlers::auth_handler;
use crate::repositories::user_repository::UserRepository;

pub fn auth_routes(repo: Arc<UserRepository>) -> Router {
    Router::new()
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login))
        .route("/auth/me", get(auth_handler::me))
        .with_state(repo)
}
