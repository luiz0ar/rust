use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use std::sync::Arc;

use crate::errors::{AppError, ErrorResponse};
use crate::models::auth::MessageResponse;
use crate::models::user::{CreateUserDto, UpdateUserDto, User};
use crate::services::{AuthService, UserService};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub auth_service: Arc<AuthService>,
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    summary = "Create user",
    description = "Registers a new user in the system.",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid request payload", body = ErrorResponse),
        (status = 409, description = "User or email already exists", body = ErrorResponse)
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    summary = "List users",
    description = "Returns a list of all registered users.",
    responses(
        (status = 200, description = "Users retrieved successfully", body = Vec<User>)
    )
)]
pub async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = state.user_service.get_users().await?;
    Ok(Json(users))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    summary = "Get user by ID",
    description = "Returns user details by ID.",
    params(
        ("id" = i64, Path, description = "Unique user ID")
    ),
    responses(
        (status = 200, description = "User found successfully", body = User),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "Users",
    summary = "Update user",
    description = "Updates an existing user's information.",
    params(
        ("id" = i64, Path, description = "Unique user ID")
    ),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated successfully", body = User),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.update_user(id, payload).await?;
    Ok(Json(user))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    summary = "Delete user",
    description = "Deletes a user from the system by ID.",
    params(
        ("id" = i64, Path, description = "Unique user ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = MessageResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    state.user_service.delete_user(id).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: format!("User {} deleted successfully", id),
        }),
    ))
}
