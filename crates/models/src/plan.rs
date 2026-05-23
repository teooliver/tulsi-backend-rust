use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PlanFilters {
    pub assigned_to: Option<Vec<Uuid>>,
    pub author: Option<Vec<Uuid>>,
    pub project_id: Option<Vec<Uuid>>,
    pub column_id: Option<Vec<Uuid>>,
    pub label_id: Option<Vec<Uuid>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub filters: Json<PlanFilters>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlanResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub filters: PlanFilters,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Plan> for PlanResponse {
    fn from(p: Plan) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            owner_id: p.owner_id,
            filters: p.filters.0,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlan {
    pub name: String,
    pub description: Option<String>,
    pub filters: PlanFilters,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlan {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filters: Option<PlanFilters>,
}
