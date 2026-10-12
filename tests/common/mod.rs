use axum::{Router, response::Response};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use src::create_app;
use src::handlers::AppState;
use src::repositories::{RefreshTokenRepository, UserRepository};
use src::services::{AuthService, UserService};

pub async fn setup_test_app() -> Router {
    setup_test_app_with_lock(std::time::Duration::from_millis(50)).await
}

pub async fn setup_test_app_with_lock(lock_duration: std::time::Duration) -> Router {
    if std::env::var("JWT_SECRET").is_err() {
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret-key-for-e2e-automated-tests");
        }
    }
    if std::env::var("LOGIN_TIME_LOCK_MS").is_err() {
        unsafe {
            std::env::set_var("LOGIN_TIME_LOCK_MS", "50");
        }
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/app_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL test database");

    sqlx::migrate!("src/database/migrations")
        .run(&pool)
        .await
        .expect("Failed to apply database migrations for tests");

    sqlx::query("TRUNCATE TABLE refresh_tokens, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate tables for testing");

    let user_repo = UserRepository::new(pool.clone());
    let refresh_token_repo = RefreshTokenRepository::new(pool);
    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        refresh_token_repo.clone(),
    ));
    let auth_service = Arc::new(
        AuthService::new(user_repo, refresh_token_repo).with_time_lock_duration(lock_duration),
    );
    let state = AppState {
        user_service,
        auth_service,
    };

    create_app(state)
}

#[allow(dead_code)]
pub async fn parse_json_response<T: DeserializeOwned>(response: Response) -> T {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read response body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("Failed to deserialize response body as JSON")
}
