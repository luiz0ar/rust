use axum::{extract::State, Json};

use crate::errors::AppError;
use crate::handlers::AppState;
use crate::models::user::{AuthResponse, LoginDto, User};
use crate::services::Claims;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.user_service.login(payload).await?;
    Ok(Json(response))
}

pub async fn me(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(claims.sub).await?;
    Ok(Json(user))
}