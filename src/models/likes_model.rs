use serde::{Deserialize, Serialize};
use sqlb::{Fields, SqlBuilder};
use sqlx::{prelude::FromRow, PgPool};

use super::{
    base::{self, DbBmc},
    ModelResult,
};

#[derive(Deserialize, Serialize, FromRow, Debug, Fields)]
pub struct LikesModel {
    pub id: i64,
    pub post_id: i64,
    pub username: String,
}

impl DbBmc for LikesModel {
    const TABLE: &'static str = "post_management.likes";
}

#[derive(Fields)]
pub struct LikePost {
    pub post_id: i64,
    pub username: String,
}

pub async fn create(pool: &PgPool, like: LikePost) -> ModelResult<i64> {
    base::create::<LikesModel, _>(like, &pool).await
}

pub async fn delete(pool: &PgPool, like: LikePost) -> ModelResult<()> {
    base::delete_with_both::<LikesModel, _, _>(
        "post_id",
        like.post_id,
        "username",
        like.username,
        &pool,
    )
    .await?;
    Ok(())
}

pub async fn get_num_likes(pool: &PgPool, post_id: i64) -> ModelResult<usize> {
    // base::count_where::<LikesModel>(pool, "id", "=", &post_id.to_string()).await
    let ids: Vec<(i64,)> = sqlb::select()
        .table(LikesModel::TABLE)
        .columns(&["id"])
        .and_where_eq("post_id", post_id)
        .fetch_all::<_, (i64,)>(pool)
        .await?;
    Ok(ids.len())
}

pub async fn is_liked(pool: &PgPool, like: LikePost) -> ModelResult<bool> {
    let found = base::get_one_with_both::<LikesModel, LikesModel, i64, &str>(
        "post_id",
        like.post_id,
        "username",
        &like.username,
        pool,
    )
    .await?
    .is_some();
    Ok(found)
}
