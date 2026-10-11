use axum::{Router, response::Response};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

use src::create_app;
use src::handlers::AppState;
use src::repositories::{RefreshTokenRepository, UserRepository};
use src::services::{AuthService, UserService};

pub async fn setup_test_app() -> Router {
    if std::env::var("JWT_SECRET").is_err() {
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret-key-for-e2e-automated-tests");
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            token_version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize test database tables");

    let user_repo = UserRepository::new(pool.clone());
    let refresh_token_repo = RefreshTokenRepository::new(pool);
    let user_service = Arc::new(UserService::new(user_repo.clone(), refresh_token_repo.clone()));
    let auth_service = Arc::new(AuthService::new(user_repo, refresh_token_repo));
    let state = AppState {
        user_service,
        auth_service,
    };

    create_app(state)
}

pub async fn parse_json_response<T: DeserializeOwned>(response: Response) -> T {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read response body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("Failed to deserialize response body as JSON")
}
