use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::task::{CreateTask, UpdateTask};
use crate::repositories::task_repository::TaskRepository;

pub async fn list_tasks(
    State(repo): State<Arc<TaskRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|tasks| Json(tasks))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_task(
    State(repo): State<Arc<TaskRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(task)) => Ok(Json(task)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_task(
    State(repo): State<Arc<TaskRepository>>,
    Json(input): Json<CreateTask>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|task| (StatusCode::CREATED, Json(task)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update_task(
    State(repo): State<Arc<TaskRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, input).await {
        Ok(Some(task)) => Ok(Json(task)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_task(
    State(repo): State<Arc<TaskRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.delete(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
