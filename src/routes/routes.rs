use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::auth_handler::{login, me};
use crate::handlers::user_handler::{
    create_user, delete_user, get_user, list_users, update_user,
};
use crate::handlers::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}
