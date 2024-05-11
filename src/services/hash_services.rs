use crate::{
    libs::hash_scheme::HashScheme,
    routes::{RouteError, RouterResult},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, Salt, SaltString},
    Argon2, PasswordVerifier,
};
use once_cell::sync::Lazy;

const ARGON2: Lazy<Argon2> = Lazy::new(get_argon2);

fn get_argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(5140, 2, 1, Some(32)).unwrap(),
    )
}

pub fn hash(password: &[u8], _scheme: &HashScheme) -> RouterResult<(String, SaltString)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = ARGON2
        .hash_password(password, &salt)
        .or(Err(RouteError::InvalidAuth))?;
    Ok((hash.to_string(), salt))
}

pub fn hash_with_salt(password: &[u8], salt: &str, _scheme: &HashScheme) -> RouterResult<String> {
    let salt = SaltString::from_b64(salt)?;
    let hash = ARGON2.hash_password(password, &salt)?;
    Ok(hash.to_string())
}

/// Returns true if password is verfied to be correct.
pub fn verify(password: &[u8], salt_str: &str, hash_str: &str) -> RouterResult<bool> {
    let mut pwd_hash = PasswordHash::new(hash_str)?;
    pwd_hash.salt = Some(Salt::from_b64(salt_str)?);
    Ok(ARGON2.verify_password(password, &pwd_hash).is_ok())
}
