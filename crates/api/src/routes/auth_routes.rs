use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use crate::handlers::auth_handler;
use repositories::user_repository::UserRepository;

/// Public auth endpoints — no token required.
pub fn auth_routes(repo: Arc<UserRepository>) -> Router {
    Router::new()
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login))
        .with_state(repo)
}

/// Authenticated auth endpoints — must run behind the `require_auth` middleware
/// so the `AuthUser` extractor can read the decoded `Claims`.
pub fn auth_protected_routes(repo: Arc<UserRepository>) -> Router {
    Router::new()
        .route("/auth/me", get(auth_handler::me))
        .with_state(repo)
}
