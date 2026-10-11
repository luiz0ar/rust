use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://sqlite.db?mode=rwc")
        .await
        .expect("Failed to connect to SQLite");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            token_version INTEGER NOT NULL DEFAULT 1
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users (email);

        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens (token);
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens (user_id);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize database table");

    // Idempotent migration for existing SQLite databases
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN token_version INTEGER NOT NULL DEFAULT 1")
        .execute(&pool)
        .await;

    pool
}
