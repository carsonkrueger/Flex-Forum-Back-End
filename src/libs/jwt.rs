use crate::{
    routes::{Error, Result},
    services::hash_services,
};
use serde::Deserialize;

use super::hash_scheme::HashScheme;

#[derive(Clone, Deserialize, Debug)]
pub struct JWT {
    id: i64,
    expires: String,
    signature: Option<String>,
}

impl JWT {
    pub fn new(id: i64, expires: String, key: &str) -> Result<JWT> {
        let mut jwt = JWT {
            id,
            expires,
            signature: None,
        };
        jwt.sign(key)?;
        Ok(jwt)
    }
    pub fn sign(&mut self, key: &str) -> Result<()> {
        let pwd = self.as_pwd();
        let hash = hash_services::hash_with(pwd.as_bytes(), key, &HashScheme::Argon2)?;
        self.signature = Some(hash);
        Ok(())
    }
    /// Parses auth_token string into its 3 parts separated by a '.'
    /// (Does not validate the hash)
    pub fn parse_token(token_str: String) -> Result<JWT> {
        let split = token_str.split(".");
        let parts: Vec<&str> = split.clone().take(3).collect();

        // split should only contain 3 different parts
        if parts.len() != 3 {
            return Err(Error::InvalidAuth);
        }

        let id = parts[0].parse::<i64>().or(Err(Error::InvalidAuth))?;

        let jwt = JWT {
            id,
            expires: parts[1].to_string(),
            signature: Some(parts[2].to_string()),
        };

        Ok(jwt)
    }
    /// Validates the jwt using the secret key and the hash, returning true if valid
    pub fn validate_token(&self, key: &str) -> Result<bool> {
        let pwd = self.as_pwd();
        hash_services::verify(
            pwd.as_bytes(),
            key,
            self.signature.as_ref().ok_or(Error::InvalidAuth)?,
        )
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn expires(&self) -> &String {
        &self.expires
    }
    pub fn signature(&self) -> &Option<String> {
        &self.signature
    }
    fn as_pwd(&self) -> String {
        format!("{}{}", self.id, self.expires)
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.id,
            self.expires,
            self.signature.clone().unwrap_or("".to_owned())
        )
    }
}
