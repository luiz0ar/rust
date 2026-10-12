use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_db() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("src/database/migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}
