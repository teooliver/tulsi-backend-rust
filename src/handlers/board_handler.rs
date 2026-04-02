use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::board::{Board, CreateBoard, UpdateBoard};
use crate::models::project::Project;
use crate::repositories::board_repository::BoardRepository;

#[utoipa::path(
    get,
    path = "/boards",
    responses(
        (status = 200, description = "List all boards", body = Vec<Board>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn list_boards(
    State(repo): State<Arc<BoardRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|boards| Json(boards))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/boards/{id}",
    params(("id" = Uuid, Path, description = "Board ID")),
    responses(
        (status = 200, description = "Board found", body = Board),
        (status = 404, description = "Board not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn get_board(
    State(repo): State<Arc<BoardRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/boards",
    request_body = CreateBoard,
    responses(
        (status = 201, description = "Board created", body = Board),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn create_board(
    State(repo): State<Arc<BoardRepository>>,
    Json(input): Json<CreateBoard>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|board| (StatusCode::CREATED, Json(board)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/boards/{id}",
    params(("id" = Uuid, Path, description = "Board ID")),
    request_body = UpdateBoard,
    responses(
        (status = 200, description = "Board updated", body = Board),
        (status = 404, description = "Board not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn update_board(
    State(repo): State<Arc<BoardRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateBoard>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, input).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/boards/{id}",
    params(("id" = Uuid, Path, description = "Board ID")),
    responses(
        (status = 204, description = "Board deleted"),
        (status = 404, description = "Board not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn delete_board(
    State(repo): State<Arc<BoardRepository>>,
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
    path = "/boards/{id}/projects",
    params(("id" = Uuid, Path, description = "Board ID")),
    responses(
        (status = 200, description = "List board's projects", body = Vec<Project>),
        (status = 404, description = "Board not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Boards"
)]
pub async fn list_board_projects(
    State(repo): State<Arc<BoardRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id(id).await {
        Ok(Some(_)) => repo
            .find_projects(id)
            .await
            .map(|projects| Json(projects))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
