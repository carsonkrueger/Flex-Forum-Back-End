use crate::{
    routes::{RouteError, RouterResult},
    services::hash_services,
};
use serde::Deserialize;
use uuid::Uuid;

use super::hash_scheme::HashScheme;

#[derive(Clone, Deserialize, Debug)]
pub struct JWT {
    id: i64,
    rng: String,
    signature: Option<String>,
}

impl JWT {
    pub fn new(id: i64, key: &str) -> RouterResult<JWT> {
        let mut jwt = JWT {
            id,
            rng: Uuid::new_v4().to_string(),
            signature: None,
        };
        jwt.sign(key)?;
        Ok(jwt)
    }
    pub fn sign(&mut self, salt: &str) -> RouterResult<()> {
        let pwd = self.as_pwd();
        let hash = hash_services::hash_with_salt(pwd.as_bytes(), salt, &HashScheme::Argon2)?;
        self.signature = Some(hash);
        Ok(())
    }
    /// Parses auth_token string into its 3 parts separated by a '.'
    /// (Does not validate the hash)
    pub fn parse_token(token_str: String) -> RouterResult<JWT> {
        let split = token_str.split(".");
        let parts: Vec<&str> = split.clone().take(3).collect();

        // split should only contain 3 different parts
        if parts.len() != 3 {
            return Err(RouteError::InvalidAuth);
        }

        let id = parts[0].parse::<i64>().or(Err(RouteError::InvalidAuth))?;

        let jwt = JWT {
            id,
            rng: parts[1].to_string(),
            signature: Some(parts[2].to_string()),
        };

        Ok(jwt)
    }
    /// Validates the jwt using the secret key and the hash, returning true if valid
    pub fn validate_token(&self, salt: &str) -> RouterResult<bool> {
        let pwd = self.as_pwd();

        hash_services::verify(
            pwd.as_bytes(),
            salt,
            self.signature.as_ref().ok_or(RouteError::InvalidAuth)?,
        )
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn rng(&self) -> &String {
        &self.rng
    }
    pub fn signature(&self) -> &Option<String> {
        &self.signature
    }
    fn as_pwd(&self) -> String {
        format!("{}{}", self.id, self.rng)
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.id,
            self.rng,
            self.signature.clone().unwrap_or("".to_owned())
        )
    }
}
