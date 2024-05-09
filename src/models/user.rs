use super::{
    base::{self, DbBmc},
    Result,
};
use crate::libs::{
    hash_scheme::HashScheme,
    validation::{RE_NAME, RE_USERNAME},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::{prelude::FromRow, PgPool};
use validator::Validate;

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserModel {
    pub id: i64,
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

impl DbBmc for UserModel {
    const TABLE: &'static str = "user_management.users";
}

#[derive(Fields)]
pub struct CreateUserModel {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
    pub pwd_salt: String,
    pub hash_scheme: HashScheme,
}

pub async fn create(pool: &PgPool, user: CreateUserModel) -> Result<i64> {
    base::create::<UserModel, _>(user, &pool).await
}

#[derive(Serialize, Validate, FromRow)]
pub struct ReadUserModel {
    // pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

pub async fn email_exists(email: &str, pool: &PgPool) -> Result<bool> {
    let result = sqlx::query("SELECT FROM user_management.users WHERE email = $2 LIMIT 1;")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(result.is_some())
}

pub async fn username_exists(username: &str, pool: &PgPool) -> Result<bool> {
    let result = sqlx::query("SELECT FROM user_management.users WHERE username = $1 LIMIT 1;")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(result.is_some())
}
