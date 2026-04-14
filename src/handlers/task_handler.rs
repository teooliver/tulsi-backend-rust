use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::task::{CreateTask, Task, UpdateTask};
use crate::models::task_history::TaskEventType;
use crate::repositories::task_history_repository::TaskHistoryRepository;
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
    auth_user: AuthUser,
    State(repo): State<Arc<TaskRepository>>,
    Extension(history_repo): Extension<Arc<TaskHistoryRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> Result<impl IntoResponse, StatusCode> {
    let old_task = match repo.find_by_id(id).await {
        Ok(Some(task)) => task,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let updated_task = match repo.update(id, input).await {
        Ok(Some(task)) => task,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let user_id = auth_user.user_id;
    record_task_changes(&history_repo, id, user_id, &old_task, &updated_task).await;

    Ok(Json(updated_task))
}

async fn record_task_changes(
    history_repo: &TaskHistoryRepository,
    task_id: Uuid,
    user_id: Uuid,
    old_task: &Task,
    new_task: &Task,
) {
    if old_task.title != new_task.title {
        if let Err(e) = history_repo
            .record(
                task_id,
                user_id,
                TaskEventType::TitleChanged,
                Some(old_task.title.clone()),
                Some(new_task.title.clone()),
            )
            .await
        {
            tracing::warn!("Failed to record title change history: {e}");
        }
    }
    if old_task.description != new_task.description {
        if let Err(e) = history_repo
            .record(
                task_id,
                user_id,
                TaskEventType::DescriptionChanged,
                Some(old_task.description.clone()),
                Some(new_task.description.clone()),
            )
            .await
        {
            tracing::warn!("Failed to record description change history: {e}");
        }
    }
    if old_task.column_id != new_task.column_id {
        if let Err(e) = history_repo
            .record(
                task_id,
                user_id,
                TaskEventType::ColumnChanged,
                old_task.column_id.map(|u| u.to_string()),
                new_task.column_id.map(|u| u.to_string()),
            )
            .await
        {
            tracing::warn!("Failed to record column change history: {e}");
        }
    }
    if old_task.assigned_to != new_task.assigned_to {
        if let Err(e) = history_repo
            .record(
                task_id,
                user_id,
                TaskEventType::AssignmentChanged,
                old_task.assigned_to.map(|u| u.to_string()),
                new_task.assigned_to.map(|u| u.to_string()),
            )
            .await
        {
            tracing::warn!("Failed to record assignment change history: {e}");
        }
    }
    if old_task.project_id != new_task.project_id {
        if let Err(e) = history_repo
            .record(
                task_id,
                user_id,
                TaskEventType::ProjectChanged,
                old_task.project_id.map(|u| u.to_string()),
                new_task.project_id.map(|u| u.to_string()),
            )
            .await
        {
            tracing::warn!("Failed to record project change history: {e}");
        }
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
