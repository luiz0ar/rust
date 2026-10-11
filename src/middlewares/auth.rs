use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::errors::AppError;
use crate::services::jwt_service::{Claims, verify_token};

#[derive(Debug, Clone)]
pub struct BearerToken(pub String);

impl<S> FromRequestParts<S> for BearerToken
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        let token = match auth_header {
            Some(auth) if auth.starts_with("Bearer ") => auth[7..].trim(),
            _ => {
                return Err(AppError::Unauthorized(
                    "Missing or malformed authorization token (use 'Bearer <token>')".to_string(),
                ));
            }
        };

        if token.is_empty() {
            return Err(AppError::Unauthorized(
                "Missing or malformed authorization token (use 'Bearer <token>')".to_string(),
            ));
        }

        Ok(BearerToken(token.to_string()))
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        let token = match auth_header {
            Some(auth) if auth.starts_with("Bearer ") => &auth[7..],
            _ => {
                return Err(AppError::Unauthorized(
                    "Missing or malformed authorization token (use 'Bearer <token>')".to_string(),
                ));
            }
        };

        verify_token(token)
    }
}
