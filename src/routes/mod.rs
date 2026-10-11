use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::AppState;
use crate::handlers::auth_handler::{login, logout, profile, refresh};
use crate::handlers::user_handler::{create_user, delete_user, get_user, list_users, update_user};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/profile", get(profile))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}
