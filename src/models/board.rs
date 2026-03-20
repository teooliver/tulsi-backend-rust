use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBoard {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBoard {
    pub name: Option<String>,
    pub description: Option<String>,
}
