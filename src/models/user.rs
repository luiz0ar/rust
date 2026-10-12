use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

fn default_token_version() -> i64 {
    1
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(default = "default_token_version")]
    pub token_version: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UserWithPassword {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(default = "default_token_version")]
    pub token_version: i64,
}
