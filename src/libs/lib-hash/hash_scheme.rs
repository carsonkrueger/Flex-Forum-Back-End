use serde::{self, Deserialize, Serialize};
use sqlb::SqlxBindable;

use crate::{
    error::HashResult,
    hashers::{argon2_v01::Argon2V01, argon2_v02::Argon2V02},
};

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "hash_scheme")]
pub enum HashScheme {
    #[sqlx(rename = "argon2_v01")]
    #[serde(rename(serialize = "argon2_v01", deserialize = "argon2_v01"))]
    Argon2V01,
    #[sqlx(rename = "argon2_v02")]
    #[serde(rename(serialize = "argon2_v02", deserialize = "argon2_v02"))]
    Argon2V02,
}

impl HashScheme {
    pub fn hasher(&self) -> Box<dyn Hasher> {
        match self {
            HashScheme::Argon2V01 => Box::new(Argon2V01),
            HashScheme::Argon2V02 => Box::new(Argon2V02),
        }
    }
}

impl SqlxBindable for HashScheme {
    fn bind_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query.bind(self)
    }
}

pub type Salt = String;
pub type Hash = String;

pub trait Hasher {
    /// Generic function that hashes a password based upon a hash_scheme, returning the hash and salt string.
    fn hash(&self, password: &str) -> HashResult<(Hash, Salt)>;
    /// Generic function that hashes a password with the given salt based upon a hash_scheme, returning the hash string.
    fn hash_with_salt(&self, password: &str, salt: &str) -> HashResult<Hash>;
    /// Generic function that validates a password, returning Ok(()) if valid.
    fn verify(&self, password: &str, salt_str: &Salt, hash_str: &Hash) -> HashResult<()>;
}
