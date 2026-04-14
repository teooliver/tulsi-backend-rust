use sqlx::PgPool;
use uuid::Uuid;

use crate::models::task_history::{TaskEventType, TaskHistory};

pub struct TaskHistoryRepository {
    pool: PgPool,
}

impl TaskHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        event_type: TaskEventType,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Result<TaskHistory, sqlx::Error> {
        sqlx::query_as::<_, TaskHistory>(
            "INSERT INTO task_history (task_id, user_id, event_type, old_value, new_value)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(task_id)
        .bind(user_id)
        .bind(&event_type)
        .bind(&old_value)
        .bind(&new_value)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_task_id(
        &self,
        task_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskHistory>, sqlx::Error> {
        sqlx::query_as::<_, TaskHistory>(
            "SELECT * FROM task_history
             WHERE task_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(task_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_task_id_and_type(
        &self,
        task_id: Uuid,
        event_type: TaskEventType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskHistory>, sqlx::Error> {
        sqlx::query_as::<_, TaskHistory>(
            "SELECT * FROM task_history
             WHERE task_id = $1 AND event_type = $2
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(task_id)
        .bind(&event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
}
