use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, Salt, SaltString},
    Argon2, PasswordHash,
};

use crate::{
    error::HashResult,
    hash_scheme::{HashScheme, Hasher},
};

pub struct Argon2V02;

impl Argon2V02 {
    fn argon2_v02<'key>() -> Argon2<'key> {
        Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(1028, 1, 1, Some(32)).unwrap(),
        )
    }
}

impl Hasher for Argon2V02 {
    fn hash(
        &self,
        password: &str,
    ) -> HashResult<(crate::hash_scheme::Hash, crate::hash_scheme::Salt)> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Self::argon2_v02().hash_password(password.as_bytes(), &salt)?;
        // .or(Err(RouteError::InvalidAuth))?;
        Ok((hash.to_string(), salt.to_string()))
    }

    fn hash_with_salt(&self, password: &str, salt: &str) -> HashResult<crate::hash_scheme::Hash> {
        let salt = SaltString::from_b64(salt)?;
        let hash = Self::argon2_v02().hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

    fn verify(
        &self,
        password: &str,
        salt_str: &crate::hash_scheme::Salt,
        hash_str: &crate::hash_scheme::Hash,
    ) -> HashResult<()> {
        let mut pwd_hash = PasswordHash::new(hash_str)?;
        pwd_hash.salt = Some(Salt::from_b64(salt_str)?);
        Self::argon2_v02().verify_password(password.as_bytes(), &pwd_hash)?;
        Ok(())
    }
}

impl From<Argon2V02> for HashScheme {
    fn from(_value: Argon2V02) -> Self {
        HashScheme::Argon2V02
    }
}
