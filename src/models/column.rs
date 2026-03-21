use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Column {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub board_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumn {
    pub name: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateColumn {
    pub name: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MoveTask {
    pub column_id: Uuid,
}
