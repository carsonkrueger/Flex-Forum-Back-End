use serde::{de::Visitor, Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(sqlx::Type)]
#[sqlx(type_name = "hash_scheme")]
pub enum HashScheme {
    #[sqlx(rename = "argon2")]
    Argon2,
}

impl PgHasArrayType for HashScheme {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("_hash_schema")
    }
}

impl From<&HashScheme> for String {
    fn from(value: &HashScheme) -> Self {
        match value {
            &HashScheme::Argon2 => "argon2".to_string(),
        }
    }
}

pub struct HashSchemeVisitor;

impl Visitor<'_> for HashSchemeVisitor {
    type Value = HashScheme;
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.to_lowercase().as_str() {
            "argon2" => Ok(HashScheme::Argon2),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &"argon2",
            )),
        }
    }
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "the string 'argon2'")
    }
}

impl<'de> Deserialize<'de> for HashScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HashSchemeVisitor)
    }
}

impl Serialize for HashScheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(&s)
    }
}
