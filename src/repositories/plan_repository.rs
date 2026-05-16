use sqlx::{PgPool, Postgres, QueryBuilder};
use sqlx::types::Json;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::plan::{CreatePlan, Plan, UpdatePlan};
use crate::models::task::Task;

pub struct PlanRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl PlanRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all_by_owner(&self, owner_id: Uuid) -> Result<Vec<Plan>, sqlx::Error> {
        let key = format!("plans:owner:{owner_id}");

        if let Some(cache) = &self.cache {
            if let Some(plans) = cache.get::<Vec<Plan>>(&key).await {
                return Ok(plans);
            }
        }

        let plans = sqlx::query_as::<_, Plan>(
            "SELECT * FROM plans WHERE owner_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &plans).await;
        }

        Ok(plans)
    }

    pub async fn find_by_id_and_owner(
        &self,
        id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<Plan>, sqlx::Error> {
        let key = format!("plan:{id}");

        if let Some(cache) = &self.cache {
            if let Some(plan) = cache.get::<Plan>(&key).await {
                if plan.owner_id == owner_id {
                    return Ok(Some(plan));
                }
            }
        }

        let plan = sqlx::query_as::<_, Plan>(
            "SELECT * FROM plans WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await?;

        if let (Some(cache), Some(plan)) = (&self.cache, &plan) {
            cache.set(&key, plan).await;
        }

        Ok(plan)
    }

    pub async fn create(&self, owner_id: Uuid, input: CreatePlan) -> Result<Plan, sqlx::Error> {
        let plan = sqlx::query_as::<_, Plan>(
            "INSERT INTO plans (name, description, owner_id, filters)
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
            cache.delete(&[&format!("plans:owner:{owner_id}")]).await;
        }

        Ok(plan)
    }

    pub async fn update(
        &self,
        id: Uuid,
        owner_id: Uuid,
        input: UpdatePlan,
    ) -> Result<Option<Plan>, sqlx::Error> {
        let plan = sqlx::query_as::<_, Plan>(
            "UPDATE plans SET
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
                    &format!("plan:{id}"),
                    &format!("plans:owner:{owner_id}"),
                ])
                .await;
        }

        Ok(plan)
    }

    pub async fn delete(&self, id: Uuid, owner_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM plans WHERE id = $1 AND owner_id = $2")
            .bind(id)
            .bind(owner_id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            cache
                .delete(&[
                    &format!("plan:{id}"),
                    &format!("plans:owner:{owner_id}"),
                ])
                .await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks_for_plan(
        &self,
        plan_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<Vec<Task>>, sqlx::Error> {
        let Some(plan) = self.find_by_id_and_owner(plan_id, owner_id).await? else {
            return Ok(None);
        };

        let filters = &plan.filters.0;
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM tasks WHERE 1=1");

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
