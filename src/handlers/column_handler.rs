use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::column::{Column, CreateColumn, MoveTask, UpdateColumn};
use crate::models::task::Task;
use crate::models::task_history::TaskEventType;
use crate::repositories::column_repository::ColumnRepository;
use crate::repositories::task_history_repository::TaskHistoryRepository;

#[utoipa::path(
    get,
    path = "/boards/{board_id}/columns",
    params(("board_id" = Uuid, Path, description = "Board ID")),
    responses(
        (status = 200, description = "List board's columns", body = Vec<Column>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn list_board_columns(
    State(repo): State<Arc<ColumnRepository>>,
    Path(board_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_by_board_id(board_id)
        .await
        .map(|columns| Json(columns))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/boards/{board_id}/columns/{column_id}",
    params(
        ("board_id" = Uuid, Path, description = "Board ID"),
        ("column_id" = Uuid, Path, description = "Column ID")
    ),
    responses(
        (status = 200, description = "Column found", body = Column),
        (status = 404, description = "Column not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn get_column(
    State(repo): State<Arc<ColumnRepository>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(column_id).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/boards/{board_id}/columns",
    params(("board_id" = Uuid, Path, description = "Board ID")),
    request_body = CreateColumn,
    responses(
        (status = 201, description = "Column created", body = Column),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn create_column(
    State(repo): State<Arc<ColumnRepository>>,
    Path(board_id): Path<Uuid>,
    Json(input): Json<CreateColumn>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(board_id, input)
        .await
        .map(|column| (StatusCode::CREATED, Json(column)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/boards/{board_id}/columns/{column_id}",
    params(
        ("board_id" = Uuid, Path, description = "Board ID"),
        ("column_id" = Uuid, Path, description = "Column ID")
    ),
    request_body = UpdateColumn,
    responses(
        (status = 200, description = "Column updated", body = Column),
        (status = 404, description = "Column not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn update_column(
    State(repo): State<Arc<ColumnRepository>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateColumn>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(column_id, input).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/boards/{board_id}/columns/{column_id}",
    params(
        ("board_id" = Uuid, Path, description = "Board ID"),
        ("column_id" = Uuid, Path, description = "Column ID")
    ),
    responses(
        (status = 204, description = "Column deleted"),
        (status = 404, description = "Column not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn delete_column(
    State(repo): State<Arc<ColumnRepository>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.delete(column_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/boards/{board_id}/columns/{column_id}/tasks",
    params(
        ("board_id" = Uuid, Path, description = "Board ID"),
        ("column_id" = Uuid, Path, description = "Column ID")
    ),
    responses(
        (status = 200, description = "List column's tasks", body = Vec<Task>),
        (status = 404, description = "Column not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn list_column_tasks(
    State(repo): State<Arc<ColumnRepository>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(column_id).await {
        Ok(Some(_)) => repo
            .find_tasks(column_id)
            .await
            .map(|tasks| Json(tasks))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    put,
    path = "/tasks/{task_id}/move",
    params(("task_id" = Uuid, Path, description = "Task ID")),
    request_body = MoveTask,
    responses(
        (status = 200, description = "Task moved", body = Task),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Columns"
)]
pub async fn move_task_to_column(
    auth_user: AuthUser,
    State(repo): State<Arc<ColumnRepository>>,
    Extension(history_repo): Extension<Arc<TaskHistoryRepository>>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<MoveTask>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.move_task(task_id, input.column_id).await {
        Ok(Some((old_column_id, task))) => {
            if old_column_id != task.column_id {
                if let Err(e) = history_repo
                    .record(
                        task_id,
                        auth_user.user_id,
                        TaskEventType::ColumnChanged,
                        old_column_id.map(|u| u.to_string()),
                        task.column_id.map(|u| u.to_string()),
                    )
                    .await
                {
                    tracing::warn!("Failed to record column change history: {e}");
                }
            }
            Ok(Json(task))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
