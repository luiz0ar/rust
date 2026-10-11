use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::handlers::AppState;
use crate::models::auth::{AuthResponse, RefreshTokenDto};
use crate::models::user::{LoginDto, User};
use crate::services::Claims;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.login(payload).await?;
    Ok(Json(response))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.refresh(payload).await?;
    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<Json<Value>, AppError> {
    state.auth_service.logout(payload).await?;
    Ok(Json(json!({
        "message": "Successfully logged out"
    })))
}

pub async fn me(claims: Claims, State(state): State<AppState>) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(claims.sub).await?;
    Ok(Json(user))
}
