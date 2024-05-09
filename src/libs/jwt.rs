use std::str::FromStr;

use serde::Deserialize;
use uuid::Uuid;

use crate::routes::{Error, Result};

#[derive(Clone, Deserialize)]
pub struct JWT {
    user_id: Uuid,
    expires: String,
    signature: String,
}

impl JWT {
    // pub fn sign(&[&str]) -> String {

    // }
    /// Parses auth_token string into its 3 parts separated by a '.'
    /// (Does not validate the hash)
    pub fn parse_token(token_str: String) -> Result<JWT> {
        let mut split = token_str.split(".");
        let parts: Vec<&str> = split.clone().take(3).collect();

        // split should only contain 3 different parts
        if parts.len() != 3 || split.next().is_some() {
            return Err(Error::InvalidAuth);
        }

        let user_id = Uuid::from_str(parts[0]).or(Err(Error::InvalidAuth))?;

        let jwt = JWT {
            user_id,
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
    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }
    pub fn expires(&self) -> &String {
        &self.expires
    }
    pub fn signature(&self) -> &String {
        &self.signature
    }
}
