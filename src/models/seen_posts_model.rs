use lib_models::error::ModelResult;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::{prelude::FromRow, PgPool};

use super::base::DbBmc;

#[derive(Deserialize, Serialize, FromRow, Debug, Fields)]
pub struct SeenPostsModel {
    pub id: i64,
    pub post_id: i64,
    pub username: String,
}

impl DbBmc for SeenPostsModel {
    const TABLE: &'static str = "post_management.seen_posts";
}

#[derive(Deserialize, Serialize, FromRow, Debug, Fields)]
pub struct SeenPostCreateModel {
    pub post_id: i64,
    pub username: String,
}

pub async fn seen(pool: &PgPool, username: &str, post_id: i64) -> ModelResult<()> {
    // let e = base::create::<SeenPostsModel, SeenPostCreateModel>(seen, pool).await;
    let _id = sqlx::query_scalar::<_, i64>(&format!(
        "INSERT INTO {} (post_id, username) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING id;",
        SeenPostsModel::TABLE
    ))
    .bind(post_id)
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(())
}
