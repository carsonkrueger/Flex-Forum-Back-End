use crate::lib::hash_scheme::HashScheme;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::time::PrimitiveDateTime;
use sqlx::FromRow;

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserModel {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub hash_scheme: HashScheme,
    pub created_at: DateTime<Utc>,
    pub deactivated_at: Option<DateTime<Utc>>,
}
