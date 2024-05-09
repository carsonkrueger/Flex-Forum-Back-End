use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2,
};
use once_cell::sync::Lazy;

use crate::{libs::hash_scheme::HashScheme, routes};

// const argon2: Argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, argon2::Params::new(m_cost, t_cost, p_cost, output_len));
pub const ARGON2: Lazy<Argon2> = Lazy::new(Argon2::default);

pub fn hash(password: &[u8]) -> Result<(String, SaltString, HashScheme), routes::Error> {
    let salt = SaltString::generate(&mut OsRng);
    // let argon2 = Argon2::default();
    // let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, argon2::Params::new(m_cost, t_cost, p_cost, output_len))
    let hash = ARGON2
        .hash_password(password, &salt)
        .or(Err(routes::Error::InvalidAuth))?;
    Ok((hash.to_string(), salt, HashScheme::Argon2))
}

pub fn hash_with<'a>(
    password: &[u8],
    salt: &'a SaltString,
) -> Result<PasswordHash<'a>, argon2::password_hash::Error> {
    let hash = ARGON2.hash_password(password, salt)?;
    Ok(hash)
}

/// Returns true if password is verfied to be correct.
pub fn verify(
    password: &[u8],
    salt_str: &str,
    hash_str: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let salt = SaltString::from_b64(salt_str)?;
    let hash = hash_with(password, &salt)?;
    return Ok(hash_str == hash.to_string());
}
