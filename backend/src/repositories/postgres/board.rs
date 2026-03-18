use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::domain::{Board, CreateBoard, DomainError, DomainResult};
use crate::repositories::BoardRepository;

pub struct PostgresBoardRepository {
    pool: PgPool,
}

impl PostgresBoardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BoardRow {
    id: i64,
    slug: String,
    name: String,
    description: Option<String>,
    thread_count: i64,
    created_at: DateTime<Utc>,
}

impl From<BoardRow> for Board {
    fn from(row: BoardRow) -> Self {
        Board {
            id: row.id,
            slug: row.slug,
            name: row.name,
            description: row.description,
            thread_count: row.thread_count as i32,
            created_at: row.created_at,
        }
    }
}

const SELECT_BOARD: &str = r#"
    SELECT
        b.id,
        b.slug,
        b.name,
        b.description,
        b.created_at,
        COUNT(p.post_number) AS thread_count
    FROM boards b
    LEFT JOIN posts p ON p.board_id = b.id AND p.parent_number IS NULL
"#;

#[async_trait]
impl BoardRepository for PostgresBoardRepository {
    async fn find_all(&self) -> DomainResult<Vec<Board>> {
        let rows = sqlx::query_as::<_, BoardRow>(&format!(
            "{} GROUP BY b.id ORDER BY b.created_at ASC",
            SELECT_BOARD
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Board::from).collect())
    }

    async fn find_by_slug(&self, slug: &str) -> DomainResult<Option<Board>> {
        let row = sqlx::query_as::<_, BoardRow>(&format!(
            "{} WHERE b.slug = $1 GROUP BY b.id",
            SELECT_BOARD
        ))
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(Board::from))
    }

    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Board>> {
        let row = sqlx::query_as::<_, BoardRow>(&format!(
            "{} WHERE b.id = $1 GROUP BY b.id",
            SELECT_BOARD
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(Board::from))
    }

    async fn create(&self, board: CreateBoard) -> DomainResult<Board> {
        // insert then fetch with thread count rather than just RETURNING
        // since a fresh board will always have 0 threads we could skip the join,
        // but using find_by_id keeps it consistent
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO boards (slug, name, description)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(&board.slug)
        .bind(&board.name)
        .bind(&board.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                DomainError::AlreadyExists(format!("board '{}' already exists", board.slug))
            }
            _ => DomainError::Database(e.to_string()),
        })?;

        // safe to unwrap - we just inserted it
        self.find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::Internal("board missing after insert".to_string()))
    }

    async fn delete_by_slug(&self, slug: &str) -> DomainResult<bool> {
        let result = sqlx::query("DELETE FROM boards WHERE slug = $1")
            .bind(slug)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, slug: &str) -> DomainResult<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM boards WHERE slug = $1)",
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(exists)
    }
}
