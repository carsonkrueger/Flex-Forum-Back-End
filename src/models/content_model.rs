use serde::{Deserialize, Serialize};
use sqlb::{Fields, SqlxBindable};
use sqlx::{prelude::FromRow, types::chrono, PgPool};

use super::{base::DbBmc, ModelResult};

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "post_type")]
pub enum PostType {
    #[sqlx(rename = "images")]
    #[serde(rename(serialize = "images", deserialize = "images"))]
    Images,
    #[sqlx(rename = "workout")]
    #[serde(rename(serialize = "workout", deserialize = "workout"))]
    Workout,
}

impl SqlxBindable for PostType {
    fn bind_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query.bind(self)
    }
}

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
pub struct ContentModel {
    pub id: i64,
    pub username: String,
    pub num_images: i16,
    pub post_type: PostType,
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
    pub post_type: PostType,
}

pub async fn get_three_older(
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
            "post_type",
            "created_at",
            "deactivated_at",
        ])
        .and_where("created_at", "<=", DateTimeUtcWrapper(created_at))
        .order_by("!created_at")
        .limit(3)
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
