use crate::errors::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User};
use crate::repositories::UserRepository;
use crate::services::password_hash::hash_password;

#[derive(Clone)]
pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
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

            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict(
                        "Email is already registered".to_string(),
                    ));
                }
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

    pub async fn delete_user(&self, id: i64) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("User with id {} not found", id)));
        }
        Ok(())
    }
}
