use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::task::Task;
use crate::models::user::{CreateUser, UpdateUser, User};

pub struct UserRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl UserRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        if let Some(cache) = &self.cache {
            if let Some(users) = cache.get::<Vec<User>>("users:all").await {
                return Ok(users);
            }
        }

        let users =
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;

        if let Some(cache) = &self.cache {
            cache.set("users:all", &users).await;
        }

        Ok(users)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let key = format!("user:{id}");

        if let Some(cache) = &self.cache {
            if let Some(user) = cache.get::<User>(&key).await {
                return Ok(Some(user));
            }
        }

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(user)) = (&self.cache, &user) {
            cache.set(&key, user).await;
        }

        Ok(user)
    }

    pub async fn create(&self, input: CreateUser) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(&input.email)
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&["users:all"]).await;
        }

        Ok(user)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET
                name = COALESCE($2, name),
                email = COALESCE($3, email),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            let key = format!("user:{id}");
            cache.delete(&[&key, "users:all"]).await;
        }

        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            let key = format!("user:{id}");
            let tasks_key = format!("user:{id}:tasks");
            cache.delete(&[&key, "users:all", &tasks_key]).await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks(&self, user_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        let key = format!("user:{user_id}:tasks");

        if let Some(cache) = &self.cache {
            if let Some(tasks) = cache.get::<Vec<Task>>(&key).await {
                return Ok(tasks);
            }
        }

        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE assigned_to = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &tasks).await;
        }

        Ok(tasks)
    }
}
