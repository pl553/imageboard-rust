use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::domain::{Admin, DomainError, DomainResult};
use crate::repositories::AdminRepository;

pub struct PostgresAdminRepository {
    pool: PgPool,
}

impl PostgresAdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AdminRow {
    id: i64,
    username: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl From<AdminRow> for Admin {
    fn from(row: AdminRow) -> Self {
        Admin {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl AdminRepository for PostgresAdminRepository {
    async fn find_by_username(&self, username: &str) -> DomainResult<Option<Admin>> {
        let result = sqlx::query_as::<_, AdminRow>(
            r#"
            SELECT id, username, password_hash, created_at
            FROM admins
            WHERE username = $1
            "#
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.map(Admin::from))
    }

    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Admin>> {
        let result = sqlx::query_as::<_, AdminRow>(
            r#"
            SELECT id, username, password_hash, created_at
            FROM admins
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.map(Admin::from))
    }

    async fn create(&self, username: &str, password_hash: &str) -> DomainResult<Admin> {
        let result = sqlx::query_as::<_, AdminRow>(
            r#"
            INSERT INTO admins (username, password_hash, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, username, password_hash, created_at
            "#
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                DomainError::AlreadyExists(format!("Admin '{}' already exists", username))
            }
            _ => DomainError::Database(e.to_string()),
        })?;

        Ok(result.into())
    }

    async fn change_password(&self, id: i64, new_password_hash: &str) -> DomainResult<()> {
        let result = sqlx::query(
            "UPDATE admins SET password_hash = $1 WHERE id = $2"
        )
        .bind(new_password_hash)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::AdminNotFound(id));
        }

        Ok(())
    }

    async fn exists_any(&self) -> DomainResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM admins LIMIT 1)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result)
    }
}
