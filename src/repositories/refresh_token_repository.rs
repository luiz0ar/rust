use sqlx::SqlitePool;

use crate::models::auth::RefreshToken;

#[derive(Clone)]
pub struct RefreshTokenRepository {
    pool: SqlitePool,
}

impl RefreshTokenRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i64,
        token: &str,
        expires_at: i64,
        created_at: i64,
    ) -> Result<RefreshToken, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();

        Ok(RefreshToken {
            id,
            user_id,
            token: token.to_string(),
            expires_at,
            created_at,
        })
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token, expires_at, created_at FROM refresh_tokens WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_latest_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token, expires_at, created_at FROM refresh_tokens WHERE user_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_by_token(&self, token: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    #[allow(dead_code)]
    pub async fn delete_by_user_id(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
