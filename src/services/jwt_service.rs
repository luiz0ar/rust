use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set")
        .into_bytes()
}

pub fn generate_token(user_id: i64, email: &str) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("System clock error: {}", e)))?
        .as_secs() as usize;

    let expiration_seconds = 24 * 60 * 60; // 24 hours
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp: now + expiration_seconds,
        iat: now,
    };

    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate JWT token: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let secret = get_jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {}", e)))?;

    Ok(token_data.claims)
}
