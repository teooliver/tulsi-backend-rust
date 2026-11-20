use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use auth::AuthUser;
use models::saved_view::{CreateSavedView, SavedViewResponse, UpdateSavedView};
use models::task::Task;
use repositories::saved_view_repository::SavedViewRepository;

#[utoipa::path(
    get,
    path = "/saved-views",
    responses(
        (status = 200, description = "List saved views for the authenticated user", body = Vec<SavedViewResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn list_saved_views(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all_by_owner(auth_user.user_id)
        .await
        .map(|saved_views| {
            let responses: Vec<SavedViewResponse> = saved_views.into_iter().map(Into::into).collect();
            Json(responses)
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/saved-views/{id}",
    params(("id" = Uuid, Path, description = "Saved View ID")),
    responses(
        (status = 200, description = "Saved View found", body = SavedViewResponse),
        (status = 404, description = "Saved View not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn get_saved_view(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id_and_owner(id, auth_user.user_id).await {
        Ok(Some(saved_view)) => Ok(Json(SavedViewResponse::from(saved_view))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/saved-views",
    request_body = CreateSavedView,
    responses(
        (status = 201, description = "Saved View created", body = SavedViewResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn create_saved_view(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
    Json(input): Json<CreateSavedView>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(auth_user.user_id, input)
        .await
        .map(|saved_view| (StatusCode::CREATED, Json(SavedViewResponse::from(saved_view))))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/saved-views/{id}",
    params(("id" = Uuid, Path, description = "Saved View ID")),
    request_body = UpdateSavedView,
    responses(
        (status = 200, description = "Saved View updated", body = SavedViewResponse),
        (status = 404, description = "Saved View not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn update_saved_view(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateSavedView>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, auth_user.user_id, input).await {
        Ok(Some(saved_view)) => Ok(Json(SavedViewResponse::from(saved_view))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/saved-views/{id}",
    params(("id" = Uuid, Path, description = "Saved View ID")),
    responses(
        (status = 204, description = "Saved View deleted"),
        (status = 404, description = "Saved View not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn delete_saved_view(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.delete(id, auth_user.user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/saved-views/{id}/tasks",
    params(("id" = Uuid, Path, description = "Saved View ID")),
    responses(
        (status = 200, description = "Tasks matching the saved view's filters", body = Vec<Task>),
        (status = 404, description = "Saved View not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Saved Views"
)]
pub async fn execute_saved_view(
    auth_user: AuthUser,
    State(repo): State<Arc<SavedViewRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_tasks_for_saved_view(id, auth_user.user_id).await {
        Ok(Some(tasks)) => Ok(Json(tasks)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
