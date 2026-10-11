use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::errors::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User};
use crate::services::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = state.user_service.get_users().await?;
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(id).await?;
    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.update_user(id, payload).await?;
    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    state.user_service.delete_user(id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": format!("User {} deleted successfully", id) })),
    ))
}
