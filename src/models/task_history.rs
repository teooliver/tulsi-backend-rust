use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "task_event_type", rename_all = "snake_case")]
pub enum TaskEventType {
    TitleChanged,
    DescriptionChanged,
    ColumnChanged,
    AssignmentChanged,
    ProjectChanged,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TaskHistory {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub event_type: TaskEventType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct HistoryQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub event_type: Option<TaskEventType>,
}
