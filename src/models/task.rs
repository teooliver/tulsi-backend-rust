use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub project_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub project_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
}
