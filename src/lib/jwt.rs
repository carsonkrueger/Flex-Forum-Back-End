use serde::Deserialize;
use uuid::Uuid;

use super::error::{Error, Result};

#[derive(Clone, Deserialize)]
pub struct JWT {
    user_id: Uuid,
    expires: String,
    signature: String,
}

impl JWT {
    pub fn parse_token(token_str: String) -> Result<JWT> {
        todo!();
        Err(Error::MissingAuthCookie)
    }
    pub fn validate_token(&self) -> Result<()> {
        todo!();
        Err(Error::InvalidAuth)
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
