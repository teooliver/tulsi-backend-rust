use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::user::{CreateUser, UpdateUser};
use crate::repositories::user_repository::UserRepository;

pub async fn list_users(
    State(repo): State<Arc<UserRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|users| Json(users))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_user(
    State(repo): State<Arc<UserRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_user(
    State(repo): State<Arc<UserRepository>>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update_user(
    State(repo): State<Arc<UserRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, input).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_user(
    State(repo): State<Arc<UserRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.delete(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_user_tasks(
    State(repo): State<Arc<UserRepository>>,
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
