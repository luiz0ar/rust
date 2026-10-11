use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::handlers::AppState;
use crate::middlewares::BearerToken;
use crate::models::auth::{AuthResponse, RefreshTokenDto};
use crate::models::user::{LoginDto, UserProfile};

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

pub async fn profile(
    bearer_token: BearerToken,
    State(state): State<AppState>,
) -> Result<Json<UserProfile>, AppError> {
    let user_profile = state.user_service.profile(&bearer_token.0).await?;
    Ok(Json(user_profile))
}