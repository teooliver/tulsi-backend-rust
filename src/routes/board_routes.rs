use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::board_handler;
use crate::repositories::board_repository::BoardRepository;

pub fn board_routes(repo: Arc<BoardRepository>) -> Router {
    Router::new()
        .route("/boards", get(board_handler::list_boards))
        .route("/boards", post(board_handler::create_board))
        .route(
            "/boards/{id}",
            get(board_handler::get_board)
                .put(board_handler::update_board)
                .delete(board_handler::delete_board),
        )
        .route(
            "/boards/{id}/projects",
            get(board_handler::list_board_projects),
        )
        .with_state(repo)
}
