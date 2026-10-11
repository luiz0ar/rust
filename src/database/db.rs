use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

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
            password TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users (email);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize database table");

    pool
}
