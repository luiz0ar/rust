mod database;
mod errors;
mod handlers;
mod middlewares;
mod models;
mod repositories;
mod routes;
mod services;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use handlers::AppState;
use repositories::{RefreshTokenRepository, UserRepository};
use services::{AuthService, UserService};

async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let host = std::env::var("HOST").expect("HOST environment variable must be set in .env");
    let port = std::env::var("PORT").expect("PORT environment variable must be set in .env");
    std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set in .env");

    let pool = database::init_db().await;

    let user_repo = UserRepository::new(pool.clone());
    let refresh_token_repo = RefreshTokenRepository::new(pool);
    let user_service = Arc::new(UserService::new(user_repo.clone()));
    let auth_service = Arc::new(AuthService::new(user_repo, refresh_token_repo));
    let state = AppState {
        user_service,
        auth_service,
    };

    let app = Router::new()
        .route("/", axum::routing::get(root))
        .merge(routes::routes())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address format for HOST:PORT");
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
