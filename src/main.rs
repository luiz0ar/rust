use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::net::SocketAddr;

#[derive(Serialize)]
struct Message {
    status: u16,
    message: String,
    user: Users,
}

#[derive(Deserialize, Serialize, FromRow)]
struct Users {
    id: Option<i64>,
    name: String,
    email: String,
    password: String,
}

async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Users>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let result = sqlx::query("INSERT INTO users (name, email, password) VALUES (?, ?, ?)")
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.password)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = Message {
        status: 201,
        message: "User created successfully".to_string(),
        user: Users {
            id: Some(result.last_insert_rowid()),
            ..payload
        },
    };

    Ok(Json(response))
}

#[derive(Serialize, FromRow)]
struct UserResponse {
    id: i64,
    name: String,
    email: String,
}

async fn get_users(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let users = sqlx::query_as::<_, UserResponse>("SELECT id, name, email FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(users))
}

async fn root() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://app.db?mode=rwc")
        .await
        .expect("Failed to connect to SQLite");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            password TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize database table");

    let app = Router::new()
        .route("/", get(root))
        .route("/user", post(create_user))
        .route("/users", get(get_users))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
