use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::label::{CreateLabel, Label, UpdateLabel};

pub struct LabelRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl LabelRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all(&self) -> Result<Vec<Label>, sqlx::Error> {
        let key = "labels:all";

        if let Some(cache) = &self.cache {
            if let Some(labels) = cache.get::<Vec<Label>>(key).await {
                return Ok(labels);
            }
        }

        let labels =
            sqlx::query_as::<_, Label>("SELECT * FROM labels ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await?;

        if let Some(cache) = &self.cache {
            cache.set(key, &labels).await;
        }

        Ok(labels)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Label>, sqlx::Error> {
        let key = format!("label:{id}");

        if let Some(cache) = &self.cache {
            if let Some(label) = cache.get::<Label>(&key).await {
                return Ok(Some(label));
            }
        }

        let label = sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(label)) = (&self.cache, &label) {
            cache.set(&key, label).await;
        }

        Ok(label)
    }

    pub async fn create(&self, input: CreateLabel) -> Result<Label, sqlx::Error> {
        let label = sqlx::query_as::<_, Label>(
            "INSERT INTO labels (name, color) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(&input.color)
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&["labels:all"]).await;
        }

        Ok(label)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateLabel,
    ) -> Result<Option<Label>, sqlx::Error> {
        let label = sqlx::query_as::<_, Label>(
            "UPDATE labels SET
                name       = COALESCE($2, name),
                color      = COALESCE($3, color),
                updated_at = NOW()
             WHERE id = $1
             RETURNING *",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.color)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache
                .delete(&[&format!("label:{id}"), "labels:all"])
                .await;
        }

        Ok(label)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM labels WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            cache
                .delete(&[&format!("label:{id}"), "labels:all"])
                .await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_labels_for_task(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Label>, sqlx::Error> {
        sqlx::query_as::<_, Label>(
            "SELECT l.*
             FROM labels l
             JOIN task_labels tl ON tl.label_id = l.id
             WHERE tl.task_id = $1
             ORDER BY l.name ASC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Returns true if the row was inserted, false if the pair already existed.
    pub async fn attach_to_task(
        &self,
        task_id: Uuid,
        label_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO task_labels (task_id, label_id)
             VALUES ($1, $2)
             ON CONFLICT DO NOTHING",
        )
        .bind(task_id)
        .bind(label_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn detach_from_task(
        &self,
        task_id: Uuid,
        label_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM task_labels WHERE task_id = $1 AND label_id = $2",
        )
        .bind(task_id)
        .bind(label_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
