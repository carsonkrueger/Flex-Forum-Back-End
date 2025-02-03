use lib_macros::{iterator_column_def, iterator_def, schema_table_def};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::model::{
    base::{self},
    error::ModelResult,
    schema::Schema,
};

#[derive(Deserialize, Serialize, FromRow, Debug)]
#[enum_def]
#[schema_table_def(Schema::PostManagement, LikesIden::Table)]
#[iterator_column_def(LikesIden)]
pub struct Likes {
    pub id: i64,
    pub post_id: i64,
    pub username: String,
}

#[iterator_column_def(LikesIden)]
#[iterator_def(LikesIden)]
pub struct LikePost {
    pub post_id: i64,
    pub username: String,
}

pub async fn get_num_likes(pool: &PgPool, post_id: i64) -> ModelResult<i64> {
    base::count_where::<Likes>(LikesIden::PostId, "=", post_id, pool).await
}

pub async fn is_liked(pool: &PgPool, post_id: i64, username: &str) -> ModelResult<bool> {
    let res = base::get_one_with_both::<Likes, Likes>(
        LikesIden::PostId,
        post_id,
        LikesIden::Username,
        username,
        pool,
    )
    .await?;
    Ok(res.is_some())
}
