use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::user::{CreateUser, UpdateUser, User};
use crate::models::task::Task;
use crate::repositories::user_repository::UserRepository;

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn list_users(
    State(repo): State<Arc<UserRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|users| Json(users))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
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

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn create_user(
    State(repo): State<Arc<UserRepository>>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
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

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
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

#[utoipa::path(
    get,
    path = "/users/{id}/tasks",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "List user's tasks", body = Vec<Task>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
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
