use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::http::StatusCode;

pub fn hash(password: &[u8]) -> Result<(String, SaltString), StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash_result = argon2.hash_password(password, &salt);
    match hash_result {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(hash) => Ok((hash.to_string(), salt)),
    }
}

pub fn verify(password: &[u8], hash: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    // let salt = Salt(hash);
    // parsed_hash.salt = Some(salt);
    Argon2::default().verify_password(password, &parsed_hash)?;
    Ok(())
}
