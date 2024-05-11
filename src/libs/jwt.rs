use crate::routes::{RouteError, RouterResult};
use uuid::Uuid;

use hash_lib::hash_scheme::Hasher;

#[derive(Clone, Debug)]
pub struct JWT {
    id: i64,
    rng: String,
    signature: Option<String>,
}

impl JWT {
    pub fn new<H: Hasher>(id: i64, key: &str, hasher: &H) -> RouterResult<JWT> {
        let mut jwt = JWT {
            id,
            rng: Uuid::new_v4().to_string(),
            signature: None,
        };
        jwt.sign(key, hasher)?;
        Ok(jwt)
    }
    pub fn sign<H: Hasher>(&mut self, salt: &str, hasher: &H) -> RouterResult<()> {
        let pwd = self.as_pwd();
        let hash = hasher.hash_with_salt(&pwd, salt)?;
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
    pub fn validate_token<H: Hasher>(&self, salt: &String, hasher: &H) -> RouterResult<()> {
        let pwd = self.as_pwd();
        hasher.verify(
            &pwd,
            salt,
            self.signature.as_ref().ok_or(RouteError::InvalidAuth)?,
        )?;
        Ok(())
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
