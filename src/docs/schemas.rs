use utoipa::OpenApi;

use crate::errors::ErrorResponse;
use crate::models::auth::{AuthResponse, MessageResponse, RefreshTokenDto};
use crate::models::user::{CreateUserDto, LoginDto, UpdateUserDto, User, UserProfile};

/// Responsabilidade única: Catálogo e registro de Schemas e DTOs OpenAPI.
#[derive(OpenApi)]
#[openapi(components(schemas(
    LoginDto,
    AuthResponse,
    RefreshTokenDto,
    MessageResponse,
    User,
    UserProfile,
    CreateUserDto,
    UpdateUserDto,
    ErrorResponse,
)))]
pub struct ApiSchemas;
