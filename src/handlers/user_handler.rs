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
    summary = "Criar usuário",
    description = "Cadastra um novo usuário no sistema.",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "Usuário criado com sucesso", body = User),
        (status = 400, description = "Dados de requisição inválidos", body = ErrorResponse),
        (status = 409, description = "Usuário ou e-mail já existente", body = ErrorResponse)
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
    summary = "Listar usuários",
    description = "Retorna a lista de todos os usuários cadastrados.",
    responses(
        (status = 200, description = "Lista de usuários retornada", body = Vec<User>)
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
    summary = "Buscar usuário por ID",
    description = "Retorna os detalhes de um usuário pelo seu ID.",
    params(
        ("id" = i64, Path, description = "ID único do usuário")
    ),
    responses(
        (status = 200, description = "Usuário encontrado com sucesso", body = User),
        (status = 404, description = "Usuário não encontrado", body = ErrorResponse)
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
    summary = "Atualizar usuário",
    description = "Atualiza os dados cadastrais de um usuário existente.",
    params(
        ("id" = i64, Path, description = "ID único do usuário")
    ),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "Usuário atualizado com sucesso", body = User),
        (status = 404, description = "Usuário não encontrado", body = ErrorResponse)
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
    summary = "Excluir usuário",
    description = "Remove um usuário do sistema pelo ID.",
    params(
        ("id" = i64, Path, description = "ID único do usuário")
    ),
    responses(
        (status = 200, description = "Usuário excluído com sucesso", body = MessageResponse),
        (status = 404, description = "Usuário não encontrado", body = ErrorResponse)
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
