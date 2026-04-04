use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::task::{CreateTask, Task, UpdateTask};

pub struct TaskRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl TaskRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all(&self) -> Result<Vec<Task>, sqlx::Error> {
        if let Some(cache) = &self.cache {
            if let Some(tasks) = cache.get::<Vec<Task>>("tasks:all").await {
                return Ok(tasks);
            }
        }

        let tasks =
            sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;

        if let Some(cache) = &self.cache {
            cache.set("tasks:all", &tasks).await;
        }

        Ok(tasks)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        let key = format!("task:{id}");

        if let Some(cache) = &self.cache {
            if let Some(task) = cache.get::<Task>(&key).await {
                return Ok(Some(task));
            }
        }

        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(task)) = (&self.cache, &task) {
            cache.set(&key, task).await;
        }

        Ok(task)
    }

    pub async fn create(&self, input: CreateTask) -> Result<Task, sqlx::Error> {
        let task = sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (title, description, project_id, assigned_to, column_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(&input.title)
        .bind(input.description.unwrap_or_default())
        .bind(input.project_id)
        .bind(input.assigned_to)
        .bind(input.column_id)
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&["tasks:all"]).await;
        }

        Ok(task)
    }

    pub async fn update(&self, id: Uuid, input: UpdateTask) -> Result<Option<Task>, sqlx::Error> {
        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                project_id = CASE WHEN $4 THEN $5 ELSE project_id END,
                assigned_to = CASE WHEN $6 THEN $7 ELSE assigned_to END,
                column_id = CASE WHEN $8 THEN $9 ELSE column_id END,
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
        .bind(input.column_id.is_some())
        .bind(input.column_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            let key = format!("task:{id}");
            cache.delete(&[&key, "tasks:all"]).await;
        }

        Ok(task)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            let key = format!("task:{id}");
            cache.delete(&[&key, "tasks:all"]).await;
        }

        Ok(result.rows_affected() > 0)
    }
}
