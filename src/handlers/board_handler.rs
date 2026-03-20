use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::board::{CreateBoard, UpdateBoard};
use crate::repositories::board_repository::BoardRepository;

pub async fn list_boards(
    State(repo): State<Arc<BoardRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all()
        .await
        .map(|boards| Json(boards))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

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

pub async fn create_board(
    State(repo): State<Arc<BoardRepository>>,
    Json(input): Json<CreateBoard>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(input)
        .await
        .map(|board| (StatusCode::CREATED, Json(board)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

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
