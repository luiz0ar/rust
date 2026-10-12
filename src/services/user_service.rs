use crate::errors::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserProfile};
use crate::repositories::{RefreshTokenRepository, UserRepository};
use crate::services::jwt_service::verify_token;
use crate::services::password_hash::hash_password;

#[derive(Clone)]
pub struct UserService {
    repo: UserRepository,
    refresh_token_repo: RefreshTokenRepository,
}

impl UserService {
    pub fn new(repo: UserRepository, refresh_token_repo: RefreshTokenRepository) -> Self {
        Self {
            repo,
            refresh_token_repo,
        }
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> Result<User, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }
        if dto.email.trim().is_empty() || !dto.email.contains('@') {
            return Err(AppError::BadRequest("Invalid email address".to_string()));
        }
        if dto.password.len() < 6 {
            return Err(AppError::BadRequest(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        if self.repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AppError::Conflict(
                "Email is already registered".to_string(),
            ));
        }

        let hashed_password = hash_password(&dto.password).map_err(AppError::Internal)?;

        let user_to_save = CreateUserDto {
            name: dto.name,
            email: dto.email,
            password: hashed_password,
        };

        let user = self.repo.create(&user_to_save).await?;
        Ok(user)
    }

    pub async fn get_users(&self) -> Result<Vec<User>, AppError> {
        let users = self.repo.find_all().await?;
        Ok(users)
    }

    /// Retrieves user data that can be accessed by other users with permission.
    /// Returns only id, name, and email (no sensitive tokens or password).
    pub async fn get_user_by_id(&self, id: i64) -> Result<User, AppError> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

        Ok(user)
    }

    pub async fn update_user(&self, id: i64, mut dto: UpdateUserDto) -> Result<User, AppError> {
        if let Some(ref email) = dto.email {
            if email.trim().is_empty() || !email.contains('@') {
                return Err(AppError::BadRequest("Invalid email address".to_string()));
            }

            if let Some(existing) = self.repo.find_by_email(email).await?
                && existing.id != id
            {
                return Err(AppError::Conflict(
                    "Email is already registered".to_string(),
                ));
            }
        }

        if let Some(ref password) = dto.password {
            if password.len() < 6 {
                return Err(AppError::BadRequest(
                    "Password must be at least 6 characters".to_string(),
                ));
            }
            dto.password = Some(hash_password(password).map_err(AppError::Internal)?);
        }

        let updated = self
            .repo
            .update(id, &dto)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

        Ok(updated)
    }

    /// Exclusive function to fetch the authenticated user's own profile using their bearer token.
    /// Returns full user data (without password), including the bearer token and their refresh token.
    pub async fn profile(&self, token: &str) -> Result<UserProfile, AppError> {
        let clean_token = token.strip_prefix("Bearer ").unwrap_or(token).trim();
        if clean_token.is_empty() {
            return Err(AppError::Unauthorized(
                "Missing or malformed authorization token (use 'Bearer <token>')".to_string(),
            ));
        }

        let claims = verify_token(clean_token)?;
        let user =
            self.repo.find_by_id(claims.sub).await?.ok_or_else(|| {
                AppError::NotFound(format!("User with id {} not found", claims.sub))
            })?;

        if claims.token_version != user.token_version {
            return Err(AppError::Unauthorized(
                "Token has been revoked or session invalidated".to_string(),
            ));
        }

        let refresh_token = self
            .refresh_token_repo
            .find_latest_by_user_id(user.id)
            .await?
            .map(|rt| rt.token);

        Ok(UserProfile {
            id: user.id,
            name: user.name,
            email: user.email,
            token: clean_token.to_string(),
            refresh_token,
        })
    }

    pub async fn delete_user(&self, id: i64) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("User with id {} not found", id)));
        }
        Ok(())
    }
}
