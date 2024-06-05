use serde::{Deserialize, Serialize};
use sqlb::{Fields, SqlxBindable};
use sqlx::{prelude::FromRow, PgPool};

use super::{
    base::{self, DbBmc},
    ModelResult,
};

#[derive(Deserialize, Serialize, FromRow)]
pub struct ContentModel {
    pub id: i64,
    pub user_id: i64,
    pub num_images: u8,
    pub description: String,
}

impl DbBmc for ContentModel {
    const TABLE: &'static str = "post_management.posts";
}

#[derive(Fields)]
pub struct CreatePostModel {
    pub username: String,
    pub num_images: i8,
    pub description: Option<String>,
}

pub async fn create<'a>(pool: &PgPool, post: CreatePostModel) -> ModelResult<i64> {
    base::create::<ContentModel, _>(post, &pool).await
}
