use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::column::{Column, CreateColumn, UpdateColumn};
use crate::models::task::Task;

pub struct ColumnRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl ColumnRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_by_board_id(&self, board_id: Uuid) -> Result<Vec<Column>, sqlx::Error> {
        let key = format!("board:{board_id}:columns");

        if let Some(cache) = &self.cache {
            if let Some(columns) = cache.get::<Vec<Column>>(&key).await {
                return Ok(columns);
            }
        }

        let columns = sqlx::query_as::<_, Column>(
            "SELECT * FROM columns WHERE board_id = $1 ORDER BY position ASC",
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &columns).await;
        }

        Ok(columns)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Column>, sqlx::Error> {
        let key = format!("column:{id}");

        if let Some(cache) = &self.cache {
            if let Some(column) = cache.get::<Column>(&key).await {
                return Ok(Some(column));
            }
        }

        let column = sqlx::query_as::<_, Column>("SELECT * FROM columns WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(column)) = (&self.cache, &column) {
            cache.set(&key, column).await;
        }

        Ok(column)
    }

    pub async fn create(
        &self,
        board_id: Uuid,
        input: CreateColumn,
    ) -> Result<Column, sqlx::Error> {
        let column = sqlx::query_as::<_, Column>(
            "INSERT INTO columns (name, position, board_id) VALUES ($1, COALESCE($2, 0), $3) RETURNING *",
        )
        .bind(&input.name)
        .bind(input.position)
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            let board_key = format!("board:{board_id}:columns");
            cache.delete(&[&board_key]).await;
        }

        Ok(column)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateColumn,
    ) -> Result<Option<Column>, sqlx::Error> {
        let column = sqlx::query_as::<_, Column>(
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
        .await?;

        if let Some(cache) = &self.cache {
            let key = format!("column:{id}");
            // Also invalidate board columns list if we have the column data
            let mut keys_to_delete = vec![key.clone()];
            if let Some(ref col) = column {
                keys_to_delete.push(format!("board:{}:columns", col.board_id));
            }
            let key_refs: Vec<&str> = keys_to_delete.iter().map(|s| s.as_str()).collect();
            cache.delete(&key_refs).await;
        }

        Ok(column)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        // Fetch the column first to know the board_id for cache invalidation
        let column = sqlx::query_as::<_, Column>("SELECT * FROM columns WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        let result = sqlx::query("DELETE FROM columns WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            let key = format!("column:{id}");
            let tasks_key = format!("column:{id}:tasks");
            let mut keys_to_delete = vec![key, tasks_key];
            if let Some(col) = column {
                keys_to_delete.push(format!("board:{}:columns", col.board_id));
            }
            let key_refs: Vec<&str> = keys_to_delete.iter().map(|s| s.as_str()).collect();
            cache.delete(&key_refs).await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks(&self, column_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        let key = format!("column:{column_id}:tasks");

        if let Some(cache) = &self.cache {
            if let Some(tasks) = cache.get::<Vec<Task>>(&key).await {
                return Ok(tasks);
            }
        }

        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE column_id = $1 ORDER BY created_at ASC",
        )
        .bind(column_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &tasks).await;
        }

        Ok(tasks)
    }

    pub async fn move_task(
        &self,
        task_id: Uuid,
        column_id: Uuid,
    ) -> Result<Option<Task>, sqlx::Error> {
        // Get the task's current column for cache invalidation
        let old_task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(&self.pool)
            .await?;

        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET column_id = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(task_id)
        .bind(column_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            let task_key = format!("task:{task_id}");
            let new_col_key = format!("column:{column_id}:tasks");
            let mut keys_to_delete = vec![task_key, new_col_key, "tasks:all".to_string()];
            if let Some(old) = old_task {
                if let Some(old_col) = old.column_id {
                    keys_to_delete.push(format!("column:{old_col}:tasks"));
                }
            }
            let key_refs: Vec<&str> = keys_to_delete.iter().map(|s| s.as_str()).collect();
            cache.delete(&key_refs).await;
        }

        Ok(task)
    }
}
