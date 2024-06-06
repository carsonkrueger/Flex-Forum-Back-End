use serde::{Deserialize, Serialize};
use sqlb::{Fields, SqlxBindable};
use sqlx::{prelude::FromRow, types::chrono, PgPool};

use super::{
    base::{self, DbBmc},
    ModelResult,
};

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct ContentModel {
    pub id: i64,
    pub username: String,
    pub num_images: i16,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deactivated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DbBmc for ContentModel {
    const TABLE: &'static str = "post_management.posts";
}

#[derive(Fields)]
pub struct CreatePostModel {
    pub username: String,
    pub num_images: i16,
    pub description: Option<String>,
}

pub async fn create(pool: &PgPool, post: CreatePostModel) -> ModelResult<i64> {
    base::create::<ContentModel, _>(post, &pool).await
}

pub async fn get_five(
    pool: &PgPool,
    created_at: &chrono::DateTime<chrono::Utc>,
) -> ModelResult<Vec<ContentModel>> {
    let row = sqlb::select()
        .table(ContentModel::TABLE)
        .columns(&[
            "id",
            "username",
            "num_images",
            "description",
            "created_at",
            "deactivated_at",
        ])
        .and_where("created_at", ">=", DateTimeUtcWrapper(created_at))
        .limit(5)
        .fetch_all::<_, ContentModel>(pool)
        .await?;
    Ok(row)
}

#[derive(Debug)]
pub struct DateTimeUtcWrapper<'a>(&'a chrono::DateTime<chrono::Utc>);

impl SqlxBindable for DateTimeUtcWrapper<'_> {
    fn bind_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query.bind(self.0)
    }
}
