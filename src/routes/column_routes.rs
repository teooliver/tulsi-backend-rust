use std::sync::Arc;

use axum::{
    Router,
    routing::{get, put},
};

use crate::handlers::column_handler;
use crate::repositories::column_repository::ColumnRepository;

pub fn column_routes(repo: Arc<ColumnRepository>) -> Router {
    Router::new()
        .route(
            "/boards/{board_id}/columns",
            get(column_handler::list_board_columns).post(column_handler::create_column),
        )
        .route(
            "/boards/{board_id}/columns/{column_id}",
            get(column_handler::get_column)
                .put(column_handler::update_column)
                .delete(column_handler::delete_column),
        )
        .route(
            "/boards/{board_id}/columns/{column_id}/tasks",
            get(column_handler::list_column_tasks),
        )
        .route(
            "/tasks/{task_id}/move",
            put(column_handler::move_task_to_column),
        )
        .with_state(repo)
}
