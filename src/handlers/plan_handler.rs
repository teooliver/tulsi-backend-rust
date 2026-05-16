use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::plan::{CreatePlan, PlanResponse, UpdatePlan};
use crate::models::task::Task;
use crate::repositories::plan_repository::PlanRepository;

#[utoipa::path(
    get,
    path = "/plans",
    responses(
        (status = 200, description = "List plans for the authenticated user", body = Vec<PlanResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn list_plans(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.find_all_by_owner(auth_user.user_id)
        .await
        .map(|plans| {
            let responses: Vec<PlanResponse> = plans.into_iter().map(Into::into).collect();
            Json(responses)
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/plans/{id}",
    params(("id" = Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Plan found", body = PlanResponse),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn get_plan(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_by_id_and_owner(id, auth_user.user_id).await {
        Ok(Some(plan)) => Ok(Json(PlanResponse::from(plan))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/plans",
    request_body = CreatePlan,
    responses(
        (status = 201, description = "Plan created", body = PlanResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn create_plan(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
    Json(input): Json<CreatePlan>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.create(auth_user.user_id, input)
        .await
        .map(|plan| (StatusCode::CREATED, Json(PlanResponse::from(plan))))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    put,
    path = "/plans/{id}",
    params(("id" = Uuid, Path, description = "Plan ID")),
    request_body = UpdatePlan,
    responses(
        (status = 200, description = "Plan updated", body = PlanResponse),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn update_plan(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePlan>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.update(id, auth_user.user_id, input).await {
        Ok(Some(plan)) => Ok(Json(PlanResponse::from(plan))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    delete,
    path = "/plans/{id}",
    params(("id" = Uuid, Path, description = "Plan ID")),
    responses(
        (status = 204, description = "Plan deleted"),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn delete_plan(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
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
    path = "/plans/{id}/tasks",
    params(("id" = Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Tasks matching the plan filters", body = Vec<Task>),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Plans"
)]
pub async fn execute_plan(
    auth_user: AuthUser,
    State(repo): State<Arc<PlanRepository>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match repo.find_tasks_for_plan(id, auth_user.user_id).await {
        Ok(Some(tasks)) => Ok(Json(tasks)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
