use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::project::{CreateProject, Project, UpdateProject};
use crate::models::task::Task;

pub struct ProjectRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl ProjectRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        if let Some(cache) = &self.cache {
            if let Some(projects) = cache.get::<Vec<Project>>("projects:all").await {
                return Ok(projects);
            }
        }

        let projects =
            sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;

        if let Some(cache) = &self.cache {
            cache.set("projects:all", &projects).await;
        }

        Ok(projects)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, sqlx::Error> {
        let key = format!("project:{id}");

        if let Some(cache) = &self.cache {
            if let Some(project) = cache.get::<Project>(&key).await {
                return Ok(Some(project));
            }
        }

        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(project)) = (&self.cache, &project) {
            cache.set(&key, project).await;
        }

        Ok(project)
    }

    pub async fn create(&self, input: CreateProject) -> Result<Project, sqlx::Error> {
        let project = sqlx::query_as::<_, Project>(
            "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(input.description.unwrap_or_default())
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&["projects:all"]).await;
        }

        Ok(project)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateProject,
    ) -> Result<Option<Project>, sqlx::Error> {
        let project = sqlx::query_as::<_, Project>(
            "UPDATE projects SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            let key = format!("project:{id}");
            cache.delete(&[&key, "projects:all"]).await;
        }

        Ok(project)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            let key = format!("project:{id}");
            let tasks_key = format!("project:{id}:tasks");
            cache.delete(&[&key, "projects:all", &tasks_key]).await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks(&self, project_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        let key = format!("project:{project_id}:tasks");

        if let Some(cache) = &self.cache {
            if let Some(tasks) = cache.get::<Vec<Task>>(&key).await {
                return Ok(tasks);
            }
        }

        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &tasks).await;
        }

        Ok(tasks)
    }
}
