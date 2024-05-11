use serde::{de::Visitor, Deserialize, Serialize};
use sqlb::SqlxBindable;
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "hash_scheme")]
pub enum HashSchemeType {
    #[sqlx(rename = "argon2")]
    Argon2,
}

impl SqlxBindable for HashSchemeType {
    fn bind_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query.bind(self)
    }
}

impl PgHasArrayType for HashSchemeType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("hash_schema")
    }
}

impl From<&HashSchemeType> for String {
    fn from(value: &HashSchemeType) -> Self {
        match value {
            &HashSchemeType::Argon2 => "argon2".to_string(),
        }
    }
}

pub struct HashSchemeVisitor;

impl Visitor<'_> for HashSchemeVisitor {
    type Value = HashSchemeType;
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.to_lowercase().as_str() {
            "argon2" => Ok(HashSchemeType::Argon2),
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

impl<'de> Deserialize<'de> for HashSchemeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HashSchemeVisitor)
    }
}

impl Serialize for HashSchemeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(&s)
    }
}
