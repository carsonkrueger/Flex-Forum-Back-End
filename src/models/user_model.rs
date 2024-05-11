use super::{
    base::{self, DbBmc},
    ModelResult,
};
use crate::libs::hash_scheme::HashScheme;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::{postgres::PgRow, prelude::FromRow, PgPool};
use validator::Validate;

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserModel {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
    pub pwd_salt: String,
    pub jwt_salt: String,
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
    // pub jwt_salt: String,
    pub hash_scheme: HashScheme,
}

pub async fn create(pool: &PgPool, user: CreateUserModel) -> ModelResult<i64> {
    base::create::<UserModel, _>(user, &pool).await
}

#[derive(Serialize, Validate, FromRow)]
pub struct ReadUserModel {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

/// Returns Some() with the email or username that is taken. None if not taken.
pub async fn username_or_email_exists(
    username: &str,
    email: &str,
    pool: &PgPool,
) -> ModelResult<Option<String>> {
    let result = sqlx::query_scalar::<_, (String, String)>("SELECT (email, username) FROM user_management.users WHERE email = $1 OR username = $2 LIMIT 1;")
        .bind(email)
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if let Some((q_email, q_name)) = result {
        if q_email == email {
            return Ok(Some(email.to_string()));
        } else if q_name == username {
            return Ok(Some(username.to_string()));
        }
    }

    Ok(None)
}

pub async fn get_one_by_username<MC, E>(username: &str, db: &PgPool) -> ModelResult<Option<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let entity = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where_eq("username", username)
        .limit(1)
        .fetch_optional(db)
        .await?;

    Ok(entity)
}
