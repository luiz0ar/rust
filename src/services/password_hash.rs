use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

pub const DUMMY_ARGON2_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$ZqTFWxHfrcuMk3OAk1aTXA$qdpwlLNyBRW1a/YOJIFHTG/4Z27WywQyN/pJMoRi+FM";

pub fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Failed to hash password: {}", e))
}

#[allow(dead_code)]
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| format!("Invalid password hash: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn dummy_verify_password(password: &str) -> bool {
    verify_password(password, DUMMY_ARGON2_HASH).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_salt_length() {
        let hash_str = hash_password("mypassword123").unwrap();
        let parsed = PasswordHash::new(&hash_str).unwrap();
        let salt_str = parsed.salt.unwrap().to_string();
        assert_eq!(salt_str.len(), 22);
    }

    #[test]
    fn test_dummy_argon2_hash_always_fails_verification() {
        assert!(!dummy_verify_password("wrong_password"));
        assert!(!dummy_verify_password(""));
    }
}
