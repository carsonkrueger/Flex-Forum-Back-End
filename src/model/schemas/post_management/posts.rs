use crate::model::error::ModelResult;
use crate::model::schema::{IntoIteratorIden, IntoSchemaTableRef};
use chrono::NaiveDateTime;
use lib_macros::{iterator_def, iterator_iden_def, schema_table_def};
use sea_query::{enum_def, Expr, PostgresQueryBuilder, Query, Value};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool, Postgres};

use crate::model::schema::Schema;

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

impl From<&PostType> for sea_query::Value {
    fn from(value: &PostType) -> Self {
        match value {
            PostType::Images => Value::String(Some(Box::new("images".to_string()))),
            PostType::Workout => Value::String(Some(Box::new("workout".to_string()))),
        }
    }
}

impl From<PostType> for sea_query::Value {
    fn from(value: PostType) -> Self {
        match &value {
            PostType::Images => Value::String(Some(Box::new("images".to_string()))),
            PostType::Workout => Value::String(Some(Box::new("workout".to_string()))),
        }
    }
}

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
#[enum_def]
#[schema_table_def(Schema::PostManagement, PostsIden::Table)]
#[iterator_iden_def(PostsIden)]
pub struct Posts {
    pub id: i64,
    pub username: String,
    pub num_images: i16,
    pub post_type: PostType,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub deactivated_at: Option<NaiveDateTime>,
}

#[iterator_iden_def(PostsIden)]
#[iterator_def(PostsIden)]
pub struct CreatePostModel {
    pub username: String,
    pub num_images: i32,
    pub description: String,
    pub post_type: PostType,
}

pub async fn get_ten_older(pool: &PgPool, created_at: &NaiveDateTime) -> ModelResult<Vec<Posts>> {
    let (sql, values) = Query::select()
        .from(Posts::schema_table_ref())
        .columns(Posts::into_iterator_iden())
        .and_where(Expr::col(PostsIden::CreatedAt).lt(*created_at))
        .order_by(PostsIden::CreatedAt, sea_query::Order::Desc)
        .limit(10)
        .build_sqlx(PostgresQueryBuilder);
    let models = sqlx::query_as_with::<Postgres, _, _>(&sql, values)
        .fetch_all(pool)
        .await?;

    Ok(models)
}

pub async fn get_ten_unseen_older<'q>(
    pool: &PgPool,
    created_at: &NaiveDateTime,
    username: &str,
) -> ModelResult<Vec<Posts>> {
    let rows = sqlx::query_as::<_, Posts>(&format!(
        "
        SELECT
            id,
            username,
            num_images,
            description,
            post_type,
            created_at,
            deactivated_at
        FROM post_management.posts p
        WHERE
            NOT EXISTS (
                SELECT 1
                FROM post_management.seen_posts s
                WHERE s.post_id = p.id
                AND s.username = $1
            )
            AND
            p.created_at < $2
        ORDER BY p.created_at DESC
        LIMIT 10;
        ",
    ))
    .bind(username)
    .bind(created_at)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
