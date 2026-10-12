use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserWithPassword};
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dto: &CreateUserDto) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email, password, token_version) VALUES ($1, $2, $3, 1) RETURNING id, name, email, token_version",
        )
        .bind(&dto.name)
        .bind(&dto.email)
        .bind(&dto.password)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, email, token_version FROM users")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, email, token_version FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserWithPassword>, sqlx::Error> {
        sqlx::query_as::<_, UserWithPassword>(
            "SELECT id, name, email, password, token_version FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn increment_token_version(&self, id: i64) -> Result<i64, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "UPDATE users SET token_version = token_version + 1 WHERE id = $1 RETURNING token_version",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(v,)| v).unwrap_or(1))
    }

    pub async fn update(&self, id: i64, dto: &UpdateUserDto) -> Result<Option<User>, sqlx::Error> {
        let current = self.find_by_id(id).await?;
        let current = match current {
            Some(u) => u,
            None => return Ok(None),
        };

        let new_name = dto.name.as_deref().unwrap_or(&current.name);
        let new_email = dto.email.as_deref().unwrap_or(&current.email);

        if let Some(ref new_password) = dto.password {
            sqlx::query("UPDATE users SET name = $1, email = $2, password = $3 WHERE id = $4")
                .bind(new_name)
                .bind(new_email)
                .bind(new_password)
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE users SET name = $1, email = $2 WHERE id = $3")
                .bind(new_name)
                .bind(new_email)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
