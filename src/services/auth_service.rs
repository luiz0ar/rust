use crate::errors::AppError;
use crate::models::auth::{AuthResponse, RefreshTokenDto};
use crate::models::user::LoginDto;
use crate::repositories::{RefreshTokenRepository, UserRepository};
use crate::services::jwt_service::{
    ACCESS_TOKEN_EXPIRATION, current_timestamp, generate_access_token, generate_refresh_token,
    refresh_token_expiration,
};
use crate::services::password_hash::verify_password;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    refresh_token_repo: RefreshTokenRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, refresh_token_repo: RefreshTokenRepository) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
        }
    }

    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        if dto.email.trim().is_empty() || dto.password.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Email and password are required".to_string(),
            ));
        }

        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        let is_valid =
            verify_password(&dto.password, &user.password).map_err(AppError::Internal)?;

        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        let access_token = generate_access_token(user.id, &user.email, user.token_version)?;
        let refresh_token = generate_refresh_token();
        let now = current_timestamp();
        let expires_at = refresh_token_expiration();

        self.refresh_token_repo
            .create(user.id, &refresh_token, expires_at, now)
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TOKEN_EXPIRATION,
        })
    }

    pub async fn refresh(&self, dto: RefreshTokenDto) -> Result<AuthResponse, AppError> {
        if dto.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Refresh token is required".to_string(),
            ));
        }

        let stored_token = self
            .refresh_token_repo
            .find_by_token(&dto.refresh_token)
            .await?
            .ok_or_else(|| {
                AppError::Unauthorized("Invalid or revoked refresh token".to_string())
            })?;

        let now = current_timestamp();
        if now > stored_token.expires_at {
            let _ = self
                .refresh_token_repo
                .delete_by_token(&dto.refresh_token)
                .await;
            return Err(AppError::Unauthorized(
                "Refresh token has expired".to_string(),
            ));
        }

        let user = self
            .user_repo
            .find_by_id(stored_token.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User no longer exists".to_string()))?;

        self.refresh_token_repo
            .delete_by_token(&dto.refresh_token)
            .await?;

        let access_token = generate_access_token(user.id, &user.email, user.token_version)?;
        let new_refresh_token = generate_refresh_token();
        let new_expires_at = refresh_token_expiration();

        self.refresh_token_repo
            .create(user.id, &new_refresh_token, new_expires_at, now)
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TOKEN_EXPIRATION,
        })
    }

    pub async fn logout(&self, dto: RefreshTokenDto) -> Result<(), AppError> {
        if dto.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Refresh token is required".to_string(),
            ));
        }

        let stored_token = self
            .refresh_token_repo
            .find_by_token(&dto.refresh_token)
            .await?;

        if let Some(token) = stored_token {
            let _ = self.user_repo.increment_token_version(token.user_id).await;
            self.refresh_token_repo
                .delete_by_token(&dto.refresh_token)
                .await?;
        }

        Ok(())
    }
}
