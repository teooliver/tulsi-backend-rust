use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::label::{CreateLabel, Label, UpdateLabel};
use crate::repositories::label_repository::LabelRepository;

fn map_create_error(e: sqlx::Error) -> StatusCode {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
            StatusCode::CONFLICT
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    get,
    path = "/labels",
    responses(
        (status = 200, description = "List all labels", body = Vec<Label>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn list_labels(
    State(repo): State<Arc<LabelRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/labels/{id}",
    params(("id" = Uuid, Path, description = "Label ID")),
    responses(
        (status = 200, description = "Label found", body = Label),
        (status = 404, description = "Label not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn get_label(
    State(repo): State<Arc<LabelRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(label)) => Ok(Json(label)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/labels",
    request_body = CreateLabel,
    responses(
        (status = 201, description = "Label created", body = Label),
        (status = 409, description = "Label name already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn create_label(
    State(repo): State<Arc<LabelRepository>>,
    Json(input): Json<CreateLabel>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|label| (StatusCode::CREATED, Json(label)))
        .map_err(map_create_error)
}

#[utoipa::path(
    put,
    path = "/labels/{id}",
    params(("id" = Uuid, Path, description = "Label ID")),
    request_body = UpdateLabel,
    responses(
        (status = 200, description = "Label updated", body = Label),
        (status = 404, description = "Label not found"),
        (status = 409, description = "Label name already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn update_label(
    State(repo): State<Arc<LabelRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateLabel>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, input).await {
        Ok(Some(label)) => Ok(Json(label)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Err(map_create_error(e)),
    }
}

#[utoipa::path(
    delete,
    path = "/labels/{id}",
    params(("id" = Uuid, Path, description = "Label ID")),
    responses(
        (status = 204, description = "Label deleted"),
        (status = 404, description = "Label not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn delete_label(
    State(repo): State<Arc<LabelRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.delete(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/tasks/{task_id}/labels",
    params(("task_id" = Uuid, Path, description = "Task ID")),
    responses(
        (status = 200, description = "Labels attached to the task", body = Vec<Label>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn list_task_labels(
    State(repo): State<Arc<LabelRepository>>,
    Path(task_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_labels_for_task(task_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    post,
    path = "/tasks/{task_id}/labels/{label_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID"),
        ("label_id" = Uuid, Path, description = "Label ID")
    ),
    responses(
        (status = 204, description = "Label attached"),
        (status = 404, description = "Task or label not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn attach_label_to_task(
    State(repo): State<Arc<LabelRepository>>,
    Path((task_id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.attach_to_task(task_id, label_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/tasks/{task_id}/labels/{label_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID"),
        ("label_id" = Uuid, Path, description = "Label ID")
    ),
    responses(
        (status = 204, description = "Label detached"),
        (status = 404, description = "Attachment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Labels"
)]
pub async fn detach_label_from_task(
    State(repo): State<Arc<LabelRepository>>,
    Path((task_id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.detach_from_task(task_id, label_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
