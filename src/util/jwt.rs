use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;

use crate::enums::permissions::Permission;

const JWT_HEADER: LazyCell<Header> = LazyCell::new(|| Header::new(Algorithm::HS256));
const VALIDATION_KEY: LazyCell<Validation> = LazyCell::new(|| Validation::new(Algorithm::HS256));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWT {
    user_id: i64,
    permissions: Vec<Permission>,
    username: String,
}

impl JWT {
    pub fn new(user_id: i64, username: String, permissions: Vec<Permission>) -> Self {
        Self {
            user_id,
            permissions,
            username,
        }
    }
    pub fn encode(&self, secret: &[u8]) -> jsonwebtoken::errors::Result<String> {
        encode(&JWT_HEADER, &self, &EncodingKey::from_secret(secret))
    }
    pub fn decode(token: &str, secret: &[u8]) -> jsonwebtoken::errors::Result<TokenData<Self>> {
        decode::<Self>(token, &DecodingKey::from_secret(secret), &VALIDATION_KEY)
    }
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }
    pub fn username(&self) -> &str {
        &self.username
    }
}
