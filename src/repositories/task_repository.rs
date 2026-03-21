use sqlx::PgPool;
use uuid::Uuid;

use crate::models::task::{CreateTask, Task, UpdateTask};

pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(&self, input: CreateTask) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (title, description, project_id, assigned_to) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(&input.title)
        .bind(input.description.unwrap_or_default())
        .bind(input.project_id)
        .bind(input.assigned_to)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, input: UpdateTask) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "UPDATE tasks SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                project_id = CASE WHEN $4 THEN $5 ELSE project_id END,
                assigned_to = CASE WHEN $6 THEN $7 ELSE assigned_to END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.project_id.is_some())
        .bind(input.project_id)
        .bind(input.assigned_to.is_some())
        .bind(input.assigned_to)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
