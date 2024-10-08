use super::base::{self, DbBmc};
use chrono::NaiveDateTime;
use lib_hash::hash_scheme::HashScheme;
use lib_models::error::ModelResult;
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::{postgres::PgRow, prelude::FromRow, PgPool};
use validator::Validate;

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct UserModel {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
    pub pwd_salt: String,
    pub hash_scheme: HashScheme,
    pub created_at: NaiveDateTime,
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

const MAX_LIMIT: i64 = 32;

pub async fn list_by_username<MC, E>(
    mut limit: i64,
    offset: i64,
    username: &str,
    db: &PgPool,
) -> ModelResult<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    limit = limit.clamp(0, MAX_LIMIT);

    let entities = base::list::<UserModel, _>(limit, offset, "username", username, db).await?;

    Ok(entities)
}

pub async fn get_user_id(username: &str, db: &PgPool) -> ModelResult<Option<i64>> {
    let id = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT id FROM {} WHERE username = $1",
        UserModel::TABLE
    ))
    .bind(username)
    .fetch_optional(db)
    .await?;
    Ok(id)
}
