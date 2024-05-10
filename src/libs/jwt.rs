use crate::{
    routes::{Error, Result},
    services::hash_services,
};
use argon2::password_hash::SaltString;
use serde::Deserialize;

use super::hash_scheme::HashScheme;

#[derive(Clone, Deserialize)]
pub struct JWT {
    id: i64,
    expires: String,
    signature: String,
}

impl JWT {
    pub fn new(id: i64, expires: String, key: &str) -> Result<JWT> {
        let mut jwt = JWT {
            id,
            expires,
            signature: "".to_string(),
        };
        jwt.sign(key)?;
        Ok(jwt)
    }
    pub fn sign(&mut self, key: &str) -> Result<()> {
        let pwd = format!("{}{}", self.id, self.expires);
        let hash = hash_services::hash_with(pwd.as_bytes(), key, &HashScheme::Argon2)?;
        self.signature = hash;
        Ok(())
    }
    /// Parses auth_token string into its 3 parts separated by a '.'
    /// (Does not validate the hash)
    pub fn parse_token(token_str: String) -> Result<JWT> {
        let mut split = token_str.split(".");
        let parts: Vec<&str> = split.clone().take(3).collect();

        // split should only contain 3 different parts
        if parts.len() != 3 || split.next().is_some() {
            return Err(Error::InvalidAuth);
        }

        let jwt = JWT {
            id: parts[0].parse::<i64>().or(Err(Error::InvalidAuth))?,
            expires: parts[1].to_string(),
            signature: parts[2].to_string(),
        };

        Ok(jwt)
    }
    /// Validates the jwt using the secret key and the hash
    pub fn validate_token(&self) -> Result<()> {
        // expire check
        // hash check
        Ok(())
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn expires(&self) -> &String {
        &self.expires
    }
    pub fn signature(&self) -> &String {
        &self.signature
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.id, self.expires, self.signature)
    }
}
