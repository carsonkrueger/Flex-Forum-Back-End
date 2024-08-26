use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::{prelude::FromRow, PgPool};

use super::{
    base::{self, DbBmc},
    ModelResult,
};

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
    let seen = SeenPostCreateModel {
        post_id,
        username: username.to_string(),
    };
    let e = base::create::<SeenPostsModel, SeenPostCreateModel>(seen, pool).await;
    println!("{:?}", e);
    Ok(())
}
