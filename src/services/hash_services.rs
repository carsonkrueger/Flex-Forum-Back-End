use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2,
};

use crate::routes;

pub fn hash(password: &[u8]) -> Result<(String, SaltString), routes::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password, &salt)
        .or(Err(routes::Error::InvalidAuth))?;
    Ok((hash.to_string(), salt))
}

pub fn hash_with<'a>(
    password: &[u8],
    salt: &'a SaltString,
) -> Result<PasswordHash<'a>, argon2::password_hash::Error> {
    let hash = Argon2::default().hash_password(password, salt)?;
    Ok(hash)
}

pub fn verify(
    password: &[u8],
    salt_str: &str,
    hash_str: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let salt = SaltString::from_b64(salt_str)?;
    let hash = hash_with(password, &salt)?;
    return Ok(hash_str == hash.to_string());
}
