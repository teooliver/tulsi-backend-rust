use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::task::{CreateTask, Task, UpdateTask};
use crate::repositories::task_repository::TaskRepository;

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List all tasks", body = Vec<Task>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tasks"
)]
pub async fn list_tasks(
    State(repo): State<Arc<TaskRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|tasks| Json(tasks))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tasks"
)]
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

#[utoipa::path(
    post,
    path = "/tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tasks"
)]
pub async fn create_task(
    auth_user: AuthUser,
    State(repo): State<Arc<TaskRepository>>,
    Json(mut input): Json<CreateTask>,
) -> Result<impl IntoResponse, StatusCode> {
    input.author = Some(auth_user.user_id);
    repo.create(input)
        .await
        .map(|task| (StatusCode::CREATED, Json(task)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    request_body = UpdateTask,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tasks"
)]
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

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    params(("id" = Uuid, Path, description = "Task ID")),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tasks"
)]
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
