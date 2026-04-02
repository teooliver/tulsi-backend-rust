use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::project::{CreateProject, Project, UpdateProject};
use crate::models::task::Task;
use crate::repositories::project_repository::ProjectRepository;

#[utoipa::path(
    get,
    path = "/projects",
    responses(
        (status = 200, description = "List all projects", body = Vec<Project>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn list_projects(
    State(repo): State<Arc<ProjectRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|projects| Json(projects))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/projects/{id}",
    params(("id" = Uuid, Path, description = "Project ID")),
    responses(
        (status = 200, description = "Project found", body = Project),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn get_project(
    State(repo): State<Arc<ProjectRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/projects",
    request_body = CreateProject,
    responses(
        (status = 201, description = "Project created", body = Project),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn create_project(
    State(repo): State<Arc<ProjectRepository>>,
    Json(input): Json<CreateProject>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|project| (StatusCode::CREATED, Json(project)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/projects/{id}",
    params(("id" = Uuid, Path, description = "Project ID")),
    request_body = UpdateProject,
    responses(
        (status = 200, description = "Project updated", body = Project),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn update_project(
    State(repo): State<Arc<ProjectRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProject>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, input).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/projects/{id}",
    params(("id" = Uuid, Path, description = "Project ID")),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn delete_project(
    State(repo): State<Arc<ProjectRepository>>,
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
    path = "/projects/{id}/tasks",
    params(("id" = Uuid, Path, description = "Project ID")),
    responses(
        (status = 200, description = "List project's tasks", body = Vec<Task>),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn list_project_tasks(
    State(repo): State<Arc<ProjectRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(_)) => repo
            .find_tasks(id)
            .await
            .map(|tasks| Json(tasks))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
