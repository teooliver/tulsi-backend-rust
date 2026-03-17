use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::project::{CreateProject, UpdateProject};
use crate::repositories::project_repository::ProjectRepository;

pub async fn list_projects(
    State(repo): State<Arc<ProjectRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|projects| Json(projects))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

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

pub async fn create_project(
    State(repo): State<Arc<ProjectRepository>>,
    Json(input): Json<CreateProject>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|project| (StatusCode::CREATED, Json(project)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

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
