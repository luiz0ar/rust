use utoipa::OpenApi;

use crate::errors::ErrorResponse;
use crate::models::auth::{AuthResponse, MessageResponse, RefreshTokenDto};
use crate::models::user::{CreateUserDto, LoginDto, UpdateUserDto, User, UserProfile};

/// Single Responsibility: Catalog and registry for OpenAPI schemas and DTOs.
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
