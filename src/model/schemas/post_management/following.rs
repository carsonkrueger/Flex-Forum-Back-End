use lib_macros::{iterator_iden_def, schema_table_def};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Pool, Postgres};

use crate::model::{
    base::{self},
    error::ModelResult,
    schema::Schema,
};

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
#[enum_def]
#[schema_table_def(Schema::UserManagement, FollowingIden::Table)]
#[iterator_iden_def(FollowingIden)]
pub struct Following {
    pub id: i64,
    pub follower: String,
    pub following: String,
}

pub async fn is_following(
    pool: &Pool<Postgres>,
    follower: &str,
    following: &str,
) -> ModelResult<bool> {
    let res = base::get_one_with_both::<Following, Following>(
        FollowingIden::Follower,
        follower,
        FollowingIden::Following,
        following,
        pool,
    )
    .await?;
    return Ok(res.is_some());
}
