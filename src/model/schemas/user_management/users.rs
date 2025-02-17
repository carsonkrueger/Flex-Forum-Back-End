use crate::model::{
    base::BaseModelTrait,
    error::ModelResult,
    schema::{FlexForumDbConnection, Schema},
};
use chrono::NaiveDateTime;
use lib_macros::{iterator_iden_def, schema_table_def};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::Validate;

#[derive(Deserialize, Serialize, FromRow, Debug)]
#[enum_def]
#[schema_table_def(Schema::UserManagement, UsersIden::Table)]
#[iterator_iden_def(UsersIden)]
pub struct Users {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
    // pub pwd_salt: String,
    // pub hash_scheme: HashScheme,
    pub created_at: NaiveDateTime,
    pub deactivated_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
#[iterator_iden_def(UsersIden)]
pub struct CreateUserModel {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
    // pub pwd_salt: String,
    // pub hash_scheme: HashScheme,
}

#[derive(Serialize, Validate, FromRow)]
#[iterator_iden_def(UsersIden)]
pub struct ReadUserModel {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

pub trait UsersModelTrait {
    async fn username_or_email_exists(
        username: &str,
        email: &str,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<String>>;
    async fn list_by_username<BM: BaseModelTrait>(
        limit: u64,
        offset: u64,
        username: &str,
        pool: &mut FlexForumDbConnection,
    ) -> ModelResult<Vec<ReadUserModel>>;
    async fn get_user_by_username<BM: BaseModelTrait>(
        username: &str,
        pool: &mut FlexForumDbConnection,
    ) -> ModelResult<Option<Users>>;
}

const MAX_LIMIT: u64 = 32;

impl UsersModelTrait for Users {
    /// Returns Some() with the email or username that is taken. None if not taken.
    async fn username_or_email_exists(
        username: &str,
        email: &str,
        pool: &mut crate::model::schema::FlexForumDbConnection,
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

    async fn list_by_username<BM: BaseModelTrait>(
        mut limit: u64,
        offset: u64,
        username: &str,
        pool: &mut FlexForumDbConnection,
    ) -> ModelResult<Vec<ReadUserModel>> {
        limit = limit.clamp(0, MAX_LIMIT);
        let entities =
            BM::list::<Users, ReadUserModel>(UsersIden::Username, username, limit, offset, pool)
                .await?;
        Ok(entities)
    }

    async fn get_user_by_username<BM: BaseModelTrait>(
        username: &str,
        pool: &mut FlexForumDbConnection,
    ) -> ModelResult<Option<Users>> {
        BM::get_one_with::<Users, Users>(UsersIden::Username, username, pool).await
    }
}
