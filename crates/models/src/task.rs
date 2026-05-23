use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub project_id: Option<Uuid>,
    pub author: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub column_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub author: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub column_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub author: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub column_id: Option<Uuid>,
}
