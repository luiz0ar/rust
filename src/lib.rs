pub mod database;
pub mod docs;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use axum::Router;
pub use handlers::AppState;
use utoipa_scalar::{Scalar, Servable};

use crate::docs::ApiDoc;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", axum::routing::get(root))
        .route("/api-docs/openapi.json", axum::routing::get(openapi_json))
        .merge(routes::routes())
        .merge(Scalar::with_url("/docs", ApiDoc::build_spec()))
        .with_state(state)
}

async fn openapi_json() -> axum::Json<utoipa::openapi::OpenApi> {
    axum::Json(ApiDoc::build_spec())
}

async fn root() -> &'static str {
    "Hello, World!"
}
