use crate::routes::{RouteError, RouterResult};

use chrono::{DateTime, Days, Utc};
use hash_lib::{error::HashError, hash_scheme::Hasher};

pub const JWT_DATE_FORMAT: &'static str = "%Y/%m/%d_%H/%M/%S_%z";

#[derive(Clone, Debug)]
pub struct JWT {
    id: i64,
    expires: DateTime<Utc>,
    signature: Option<String>,
}

#[allow(unused)]
impl JWT {
    pub fn new<H: Hasher>(id: i64, key: &str, hasher: &H) -> RouterResult<JWT> {
        let expires = Utc::now().checked_add_days(Days::new(1)).unwrap();
        // expires.with_timezone(TimeZone:)

        let mut jwt = JWT {
            id,
            expires,
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

        let expires =
            chrono::DateTime::parse_from_str(&parts[1].to_string(), JWT_DATE_FORMAT)?.to_utc();

        let jwt = JWT {
            id,
            expires,
            signature: Some(parts[2].to_string()),
        };

        Ok(jwt)
    }
    /// Validates the jwt using the secret key and the hash, returning true if valid
    pub fn validate_token<H: Hasher>(&self, salt: &String, hasher: &H) -> RouterResult<()> {
        let now = chrono::Utc::now();
        if self.expires <= now {
            return Err(RouteError::ExpiredAuthToken);
        }

        let pwd = self.as_pwd();
        let hash_result = hasher.verify(
            &pwd,
            salt,
            self.signature
                .as_ref()
                .ok_or(RouteError::MissingJWTSignature)?,
        );

        match hash_result {
            Err(HashError::Argon2Error(argon2::password_hash::Error::Password)) => {
                return Err(RouteError::InvalidAuth)
            }
            Err(_) => return Err(RouteError::HashError),
            _ => (),
        }

        Ok(())
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
    pub fn expires(&self) -> &DateTime<Utc> {
        &self.expires
    }
    pub fn signature(&self) -> &Option<String> {
        &self.signature
    }
    fn as_pwd(&self) -> String {
        format!("{}{}", self.id, self.expires_to_string())
    }
    fn expires_to_string(&self) -> String {
        self.expires().format(&JWT_DATE_FORMAT).to_string()
    }
}

impl ToString for JWT {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.id,
            self.expires_to_string(),
            self.signature.clone().unwrap_or("".to_owned())
        )
    }
}
