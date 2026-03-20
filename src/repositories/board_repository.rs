use sqlx::PgPool;
use uuid::Uuid;

use crate::models::board::{Board, CreateBoard, UpdateBoard};
use crate::models::project::Project;

pub struct BoardRepository {
    pool: PgPool,
}

impl BoardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>("SELECT * FROM boards ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>("SELECT * FROM boards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(&self, input: CreateBoard) -> Result<Board, sqlx::Error> {
        sqlx::query_as::<_, Board>(
            "INSERT INTO boards (name, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(input.description.unwrap_or_default())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateBoard,
    ) -> Result<Option<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>(
            "UPDATE boards SET
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
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM boards WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_projects(&self, board_id: Uuid) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE board_id = $1 ORDER BY created_at DESC",
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await
    }
}
