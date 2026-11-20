use sqlx::{PgPool, Postgres, QueryBuilder};
use sqlx::types::Json;
use uuid::Uuid;

use cache::RedisCache;
use models::saved_view::{CreateSavedView, SavedView, UpdateSavedView};
use models::task::Task;

pub struct SavedViewRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl SavedViewRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all_by_owner(&self, owner_id: Uuid) -> Result<Vec<SavedView>, sqlx::Error> {
        let key = format!("saved_views:owner:{owner_id}");

        if let Some(cache) = &self.cache {
            if let Some(saved_views) = cache.get::<Vec<SavedView>>(&key).await {
                return Ok(saved_views);
            }
        }

        let saved_views = sqlx::query_as::<_, SavedView>(
            "SELECT * FROM saved_views WHERE owner_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &saved_views).await;
        }

        Ok(saved_views)
    }

    pub async fn find_by_id_and_owner(
        &self,
        id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<SavedView>, sqlx::Error> {
        let key = format!("saved_view:{id}");

        if let Some(cache) = &self.cache {
            if let Some(saved_view) = cache.get::<SavedView>(&key).await {
                if saved_view.owner_id == owner_id {
                    return Ok(Some(saved_view));
                }
            }
        }

        let saved_view = sqlx::query_as::<_, SavedView>(
            "SELECT * FROM saved_views WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await?;

        if let (Some(cache), Some(saved_view)) = (&self.cache, &saved_view) {
            cache.set(&key, saved_view).await;
        }

        Ok(saved_view)
    }

    pub async fn create(&self, owner_id: Uuid, input: CreateSavedView) -> Result<SavedView, sqlx::Error> {
        let saved_view = sqlx::query_as::<_, SavedView>(
            "INSERT INTO saved_views (name, description, owner_id, filters)
             VALUES ($1, $2, $3, $4)
             RETURNING *",
        )
        .bind(&input.name)
        .bind(input.description.unwrap_or_default())
        .bind(owner_id)
        .bind(Json(&input.filters))
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&[&format!("saved_views:owner:{owner_id}")]).await;
        }

        Ok(saved_view)
    }

    pub async fn update(
        &self,
        id: Uuid,
        owner_id: Uuid,
        input: UpdateSavedView,
    ) -> Result<Option<SavedView>, sqlx::Error> {
        let saved_view = sqlx::query_as::<_, SavedView>(
            "UPDATE saved_views SET
                name        = COALESCE($3, name),
                description = COALESCE($4, description),
                filters     = CASE WHEN $5 THEN $6 ELSE filters END,
                updated_at  = NOW()
             WHERE id = $1 AND owner_id = $2
             RETURNING *",
        )
        .bind(id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.filters.is_some())
        .bind(input.filters.map(Json))
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache
                .delete(&[
                    &format!("saved_view:{id}"),
                    &format!("saved_views:owner:{owner_id}"),
                ])
                .await;
        }

        Ok(saved_view)
    }

    pub async fn delete(&self, id: Uuid, owner_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM saved_views WHERE id = $1 AND owner_id = $2")
            .bind(id)
            .bind(owner_id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            cache
                .delete(&[
                    &format!("saved_view:{id}"),
                    &format!("saved_views:owner:{owner_id}"),
                ])
                .await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks_for_saved_view(
        &self,
        saved_view_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<Vec<Task>>, sqlx::Error> {
        let Some(saved_view) = self.find_by_id_and_owner(saved_view_id, owner_id).await? else {
            return Ok(None);
        };

        let filters = &saved_view.filters.0;
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM tasks WHERE deleted_at IS NULL");

        if let Some(ids) = &filters.assigned_to {
            if !ids.is_empty() {
                qb.push(" AND assigned_to = ANY(");
                qb.push_bind(ids.as_slice());
                qb.push(")");
            }
        }

        if let Some(ids) = &filters.author {
            if !ids.is_empty() {
                qb.push(" AND author = ANY(");
                qb.push_bind(ids.as_slice());
                qb.push(")");
            }
        }

        if let Some(ids) = &filters.project_id {
            if !ids.is_empty() {
                qb.push(" AND project_id = ANY(");
                qb.push_bind(ids.as_slice());
                qb.push(")");
            }
        }

        if let Some(ids) = &filters.column_id {
            if !ids.is_empty() {
                qb.push(" AND column_id = ANY(");
                qb.push_bind(ids.as_slice());
                qb.push(")");
            }
        }

        if let Some(ids) = &filters.label_id {
            if !ids.is_empty() {
                qb.push(" AND id IN (SELECT task_id FROM task_labels WHERE label_id = ANY(");
                qb.push_bind(ids.as_slice());
                qb.push("))");
            }
        }

        if let Some(after) = filters.created_after {
            qb.push(" AND created_at >= ");
            qb.push_bind(after);
        }

        if let Some(before) = filters.created_before {
            qb.push(" AND created_at <= ");
            qb.push_bind(before);
        }

        qb.push(" ORDER BY created_at DESC");

        let tasks = qb.build_query_as::<Task>().fetch_all(&self.pool).await?;
        Ok(Some(tasks))
    }
}
