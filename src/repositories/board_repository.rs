use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::board::{Board, CreateBoard, UpdateBoard};
use crate::models::project::Project;

pub struct BoardRepository {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl BoardRepository {
    pub fn new(pool: PgPool, cache: Option<RedisCache>) -> Self {
        Self { pool, cache }
    }

    pub async fn find_all(&self) -> Result<Vec<Board>, sqlx::Error> {
        if let Some(cache) = &self.cache {
            if let Some(boards) = cache.get::<Vec<Board>>("boards:all").await {
                return Ok(boards);
            }
        }

        let boards =
            sqlx::query_as::<_, Board>("SELECT * FROM boards ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;

        if let Some(cache) = &self.cache {
            cache.set("boards:all", &boards).await;
        }

        Ok(boards)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Board>, sqlx::Error> {
        let key = format!("board:{id}");

        if let Some(cache) = &self.cache {
            if let Some(board) = cache.get::<Board>(&key).await {
                return Ok(Some(board));
            }
        }

        let board = sqlx::query_as::<_, Board>("SELECT * FROM boards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let (Some(cache), Some(board)) = (&self.cache, &board) {
            cache.set(&key, board).await;
        }

        Ok(board)
    }

    pub async fn create(&self, input: CreateBoard) -> Result<Board, sqlx::Error> {
        let board = sqlx::query_as::<_, Board>(
            "INSERT INTO boards (name, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(&input.name)
        .bind(input.description.unwrap_or_default())
        .fetch_one(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.delete(&["boards:all"]).await;
        }

        Ok(board)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: UpdateBoard,
    ) -> Result<Option<Board>, sqlx::Error> {
        let board = sqlx::query_as::<_, Board>(
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
        .await?;

        if let Some(cache) = &self.cache {
            let key = format!("board:{id}");
            cache.delete(&[&key, "boards:all"]).await;
        }

        Ok(board)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM boards WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(cache) = &self.cache {
            let key = format!("board:{id}");
            let projects_key = format!("board:{id}:projects");
            cache.delete(&[&key, "boards:all", &projects_key]).await;
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_projects(&self, board_id: Uuid) -> Result<Vec<Project>, sqlx::Error> {
        let key = format!("board:{board_id}:projects");

        if let Some(cache) = &self.cache {
            if let Some(projects) = cache.get::<Vec<Project>>(&key).await {
                return Ok(projects);
            }
        }

        let projects = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE board_id = $1 ORDER BY created_at DESC",
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(cache) = &self.cache {
            cache.set(&key, &projects).await;
        }

        Ok(projects)
    }
}
