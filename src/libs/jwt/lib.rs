pub mod error;

use chrono::{DateTime, TimeDelta, Utc};
use lib_hash::{error::HashError, hash_scheme::HashScheme};

use crate::error::{JWTError, JWTResult};

pub const JWT_DATE_FORMAT: &'static str = "%Y/%m/%d_%H/%M/%S_%z";
pub const JWT_LIFE_IN_MINUTES: i64 = 60;

#[derive(Clone, Debug)]
pub struct JWT {
    username: String,
    expires: DateTime<Utc>,
    signature: Option<String>,
}

#[allow(unused)]
impl JWT {
    const JWT_HASH_SCHEME: HashScheme = HashScheme::Argon2V02;
    pub fn new(username: String, key: &str) -> JWTResult<JWT> {
        let expires = Utc::now()
            .checked_add_signed(TimeDelta::minutes(JWT_LIFE_IN_MINUTES))
            .unwrap();

        let mut jwt = JWT {
            username,
            expires,
            signature: None,
        };

        jwt.sign(key)?;

        Ok(jwt)
    }
    pub fn sign(&mut self, salt: &str) -> JWTResult<()> {
        let pwd = self.as_pwd();
        let hasher = Self::JWT_HASH_SCHEME.hasher();
        let hash = hasher.hash_with_salt(&pwd, salt)?;
        let hash_end = hash.strip_prefix(hasher.hash_prefix()).unwrap().to_string();
        self.signature = Some(hash_end);
        Ok(())
    }
    /// Parses auth_token string into its 3 parts separated by a '.'
    /// (Does not validate the hash)
    pub fn parse_token(token_str: String) -> JWTResult<JWT> {
        let split = token_str.split(".");
        let parts: Vec<&str> = split.clone().take(3).collect();

        // split should only contain 3 different parts
        if parts.len() != 3 {
            return Err(JWTError::InvalidJWT);
        }

        let username = parts[0].to_owned();

        let expires =
            chrono::DateTime::parse_from_str(&parts[1].to_string(), JWT_DATE_FORMAT)?.to_utc();

        let hasher = Self::JWT_HASH_SCHEME.hasher();
        let signature = Some(format!("{}{}", hasher.hash_prefix(), parts[2]));

        let jwt = JWT {
            username,
            expires,
            signature,
        };

        Ok(jwt)
    }
    /// Validates the jwt using the secret key and the hash, returning true if valid
    pub fn validate_token(&self, salt: &String) -> JWTResult<()> {
        let now = chrono::Utc::now();
        if self.expires <= now {
            return Err(JWTError::ExpiredJWT);
        }

        let pwd = self.as_pwd();
        let hasher = Self::JWT_HASH_SCHEME.hasher();
        let hash_result = hasher.verify(
            &pwd,
            salt,
            self.signature
                .as_ref()
                .ok_or(JWTError::MissingJWTSignature)?,
        );

        match hash_result {
            Err(HashError::Argon2Error(argon2::password_hash::Error::Password)) => {
                return Err(JWTError::InvalidJWT)
            }
            Err(_) => return Err(JWTError::HashError),
            _ => (),
        }

        Ok(())
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn expires(&self) -> &DateTime<Utc> {
        &self.expires
    }
    pub fn signature(&self) -> &Option<String> {
        &self.signature
    }
    fn as_pwd(&self) -> String {
        let pwd = format!("{}{}", self.username, self.expires_to_string());
        pwd
    }
    fn expires_to_string(&self) -> String {
        self.expires().format(&JWT_DATE_FORMAT).to_string()
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.username,
            self.expires_to_string(),
            self.signature.clone().unwrap_or("".to_owned())
        )
    }
}
