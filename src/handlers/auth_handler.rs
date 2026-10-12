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
    summary = "Login de usuário",
    description = "Autentica usuário por e-mail e senha, retornando tokens JWT.",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login efetuado com sucesso", body = AuthResponse),
        (status = 401, description = "Credenciais inválidas", body = ErrorResponse)
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
    summary = "Renovação de tokens",
    description = "Emite um novo par de tokens a partir de um refresh token válido.",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Tokens renovados com sucesso", body = AuthResponse),
        (status = 401, description = "Refresh token inválido ou expirado", body = ErrorResponse)
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
    summary = "Logout de usuário",
    description = "Revoga o refresh token e encerra a sessão ativa.",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Logout efetuado com sucesso", body = MessageResponse),
        (status = 401, description = "Token inválido", body = ErrorResponse)
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
    summary = "Perfil do usuário",
    description = "Retorna os dados do perfil do usuário autenticado via Bearer Token.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Perfil retornado com sucesso", body = UserProfile),
        (status = 401, description = "Token ausente ou inválido", body = ErrorResponse),
        (status = 404, description = "Usuário não encontrado", body = ErrorResponse)
    )
)]
pub async fn profile(
    bearer_token: BearerToken,
    State(state): State<AppState>,
) -> Result<Json<UserProfile>, AppError> {
    let user_profile = state.user_service.profile(&bearer_token.0).await?;
    Ok(Json(user_profile))
}
