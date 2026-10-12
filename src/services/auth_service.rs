use std::time::Duration;

use tokio::time::Instant;

use crate::errors::AppError;
use crate::models::auth::{AuthResponse, RefreshTokenDto};
use crate::models::user::LoginDto;
use crate::repositories::{RefreshTokenRepository, UserRepository};
use crate::services::jwt_service::{
    ACCESS_TOKEN_EXPIRATION, current_timestamp, generate_access_token, generate_refresh_token,
    refresh_token_expiration,
};
use crate::services::password_hash::{DUMMY_ARGON2_HASH, verify_password};

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    refresh_token_repo: RefreshTokenRepository,
    time_lock_duration: Duration,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, refresh_token_repo: RefreshTokenRepository) -> Self {
        let time_lock_ms: u64 = std::env::var("LOGIN_TIME_LOCK_MS")
            .expect("LOGIN_TIME_LOCK_MS environment variable must be set in .env")
            .parse()
            .expect("LOGIN_TIME_LOCK_MS must be a valid integer");

        Self {
            user_repo,
            refresh_token_repo,
            time_lock_duration: Duration::from_millis(time_lock_ms),
        }
    }

    pub fn with_time_lock_duration(mut self, duration: Duration) -> Self {
        self.time_lock_duration = duration;
        self
    }

    pub fn time_lock_duration(&self) -> Duration {
        self.time_lock_duration
    }

    /// Authenticates user and enforces a constant-time response lock.
    /// Regardless of whether the login succeeds or fails (invalid password, unknown email,
    /// empty fields, etc.), the response will only be returned after the configured time lock window.
    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        let start = Instant::now();
        let result = self.execute_login(dto).await;

        let elapsed = start.elapsed();
        if elapsed < self.time_lock_duration {
            tokio::time::sleep(self.time_lock_duration - elapsed).await;
        }

        result
    }

    async fn execute_login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        if dto.email.trim().is_empty() || dto.password.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Email and password are required".to_string(),
            ));
        }

        let user_opt = self.user_repo.find_by_email(&dto.email).await?;

        let (user, password_hash) = match user_opt {
            Some(user) => {
                let hash = user.password.clone();
                (Some(user), hash)
            }
            None => (None, DUMMY_ARGON2_HASH.to_string()),
        };

        let is_valid = verify_password(&dto.password, &password_hash).unwrap_or(false);

        let user = match user {
            Some(u) if is_valid => u,
            _ => {
                return Err(AppError::Unauthorized(
                    "Invalid email or password".to_string(),
                ));
            }
        };

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
