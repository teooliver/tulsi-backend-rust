use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::column::{CreateColumn, MoveTask, UpdateColumn};
use crate::repositories::column_repository::ColumnRepository;

pub async fn list_board_columns(
    State(repo): State<Arc<ColumnRepository>>,
    Path(board_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_by_board_id(board_id)
        .await
        .map(|columns| Json(columns))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

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

pub async fn move_task_to_column(
    State(repo): State<Arc<ColumnRepository>>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<MoveTask>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.move_task(task_id, input.column_id).await {
        Ok(Some(task)) => Ok(Json(task)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
