use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::handlers::AppState;
use crate::handlers::auth_handler::{login, logout, profile, refresh};
use crate::handlers::user_handler::{create_user, delete_user, get_user, list_users, update_user};
use crate::middlewares::login_time_lock_middleware;

pub fn routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/login",
            post(login).route_layer(from_fn_with_state(
                state.clone(),
                login_time_lock_middleware,
            )),
        )
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/profile", get(profile))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}
