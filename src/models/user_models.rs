use crate::lib::hash_scheme::HashScheme;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::time::PrimitiveDateTime};
use uuid::Uuid;
// use sqlx::types::Uuid;

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub hash_scheme: HashScheme,
    pub created_at: chrono::NaiveDate,
    pub deactivated_at: Option<NaiveDateTime>,
}
