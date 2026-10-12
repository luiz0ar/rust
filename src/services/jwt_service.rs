use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub email: String,
    pub token_version: i64,
    pub exp: usize,
    pub iat: usize,
}

pub const ACCESS_TOKEN_EXPIRATION: usize = 15 * 60; // 15 minutes - 900 seconds
pub const REFRESH_TOKEN_EXPIRATION_DAYS: i64 = 7; // 7 days - 604800 seconds

fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set")
        .into_bytes()
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn refresh_token_expiration() -> i64 {
    current_timestamp() + (REFRESH_TOKEN_EXPIRATION_DAYS * 24 * 60 * 60)
}

pub fn generate_refresh_token() -> String {
    let random_bytes: [u8; 32] = rand::random();
    random_bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn generate_access_token(
    user_id: i64,
    email: &str,
    token_version: i64,
) -> Result<String, AppError> {
    let now = current_timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        token_version,
        exp: now + ACCESS_TOKEN_EXPIRATION,
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

#[allow(dead_code)]
pub fn generate_token(user_id: i64, email: &str, token_version: i64) -> Result<String, AppError> {
    generate_access_token(user_id, email, token_version)
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
