use crate::lib::hash_scheme::HashScheme;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::time::PrimitiveDateTime;

#[derive(Deserialize, Serialize)]
pub struct UserModel {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub hash_scheme: HashScheme,
    pub created_at: NaiveDateTime,
    pub deactivated_at: Option<NaiveDateTime>,
}
