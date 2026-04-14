use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::task_history::{HistoryQueryParams, TaskHistory};
use crate::repositories::task_history_repository::TaskHistoryRepository;

#[utoipa::path(
    get,
    path = "/tasks/{id}/history",
    params(
        ("id" = Uuid, Path, description = "Task ID"),
        HistoryQueryParams,
    ),
    responses(
        (status = 200, description = "Task history", body = Vec<TaskHistory>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Task History"
)]
pub async fn get_task_history(
    Extension(repo): Extension<Arc<TaskHistoryRepository>>,
    Path(task_id): Path<Uuid>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let history = match params.event_type {
        Some(event_type) => {
            repo.find_by_task_id_and_type(task_id, event_type, limit, offset)
                .await
        }
        None => repo.find_by_task_id(task_id, limit, offset).await,
    };

    history
        .map(|h| Json(h))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
