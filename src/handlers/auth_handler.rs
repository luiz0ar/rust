use axum::{Json, extract::State};

use crate::errors::{AppError, ErrorResponse};
use crate::handlers::AppState;
use crate::middlewares::BearerToken;
use crate::models::auth::{AuthResponse, MessageResponse, RefreshTokenDto};
use crate::models::user::{LoginDto, UserProfile};

#[utoipa::path(
    post,
    path = "/login",
    tag = "Auth",
    summary = "User login",
    description = "Authenticates user with email and password, returning JWT access and refresh tokens.",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.login(payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/refresh",
    tag = "Auth",
    summary = "Refresh tokens",
    description = "Issues a new token pair using a valid refresh token.",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse)
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.refresh(payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "Auth",
    summary = "User logout",
    description = "Revokes the refresh token and ends the active session.",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Logout successful", body = MessageResponse),
        (status = 401, description = "Invalid token", body = ErrorResponse)
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<Json<MessageResponse>, AppError> {
    state.auth_service.logout(payload).await?;
    Ok(Json(MessageResponse {
        message: "Successfully logged out".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/profile",
    tag = "Auth",
    summary = "User profile",
    description = "Returns profile details for the authenticated user via Bearer token.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Profile retrieved successfully", body = UserProfile),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn profile(
    bearer_token: BearerToken,
    State(state): State<AppState>,
) -> Result<Json<UserProfile>, AppError> {
    let user_profile = state.user_service.profile(&bearer_token.0).await?;
    Ok(Json(user_profile))
}
