use std::sync::Arc;

use axum::{
    Router,
    routing::get,
};

use crate::handlers::plan_handler;
use crate::repositories::plan_repository::PlanRepository;

pub fn plan_routes(repo: Arc<PlanRepository>) -> Router {
    Router::new()
        .route("/plans", get(plan_handler::list_plans).post(plan_handler::create_plan))
        .route(
            "/plans/{id}",
            get(plan_handler::get_plan)
                .put(plan_handler::update_plan)
                .delete(plan_handler::delete_plan),
        )
        .route("/plans/{id}/tasks", get(plan_handler::execute_plan))
        .with_state(repo)
}
