use sqlx::PgPool;
use uuid::Uuid;

use crate::models::column::{Column, CreateColumn, UpdateColumn};
use crate::models::task::Task;

pub struct ColumnRepository {
    pool: PgPool,
}

impl ColumnRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_board_id(&self, board_id: Uuid) -> Result<Vec<Column>, sqlx::Error> {
        sqlx::query_as::<_, Column>(
            "SELECT * FROM columns WHERE board_id = $1 ORDER BY position ASC",
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Column>, sqlx::Error> {
        sqlx::query_as::<_, Column>("SELECT * FROM columns WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        board_id: Uuid,
        input: CreateColumn,
    ) -> Result<Column, sqlx::Error> {
        sqlx::query_as::<_, Column>(
            "INSERT INTO columns (name, position, board_id) VALUES ($1, COALESCE($2, 0), $3) RETURNING *",
        )
        .bind(&input.name)
        .bind(input.position)
        .bind(board_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateColumn,
    ) -> Result<Option<Column>, sqlx::Error> {
        sqlx::query_as::<_, Column>(
            "UPDATE columns SET
                name = COALESCE($2, name),
                position = COALESCE($3, position),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.position)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM columns WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks(&self, column_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE column_id = $1 ORDER BY created_at ASC",
        )
        .bind(column_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn move_task(
        &self,
        task_id: Uuid,
        column_id: Uuid,
    ) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "UPDATE tasks SET column_id = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(task_id)
        .bind(column_id)
        .fetch_optional(&self.pool)
        .await
    }
}
