use sqlx::PgPool;
use uuid::Uuid;

use crate::models::task::Task;
use crate::models::user::{CreateUser, UpdateUser, User};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(&self, input: CreateUser) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(&input.email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
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
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_tasks(&self, user_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE assigned_to = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
}
