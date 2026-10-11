pub mod database;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use axum::Router;
pub use handlers::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", axum::routing::get(root))
        .merge(routes::routes())
        .with_state(state)
}

async fn root() -> &'static str {
    "Hello, World!"
}
