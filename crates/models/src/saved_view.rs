use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct SavedViewFilters {
    pub assigned_to: Option<Vec<Uuid>>,
    pub author: Option<Vec<Uuid>>,
    pub project_id: Option<Vec<Uuid>>,
    pub column_id: Option<Vec<Uuid>>,
    pub label_id: Option<Vec<Uuid>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SavedView {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub filters: Json<SavedViewFilters>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SavedViewResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub filters: SavedViewFilters,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SavedView> for SavedViewResponse {
    fn from(p: SavedView) -> Self {
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
pub struct CreateSavedView {
    pub name: String,
    pub description: Option<String>,
    pub filters: SavedViewFilters,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSavedView {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filters: Option<SavedViewFilters>,
}
