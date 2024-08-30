use chrono::NaiveDateTime;
use itertools::Itertools;
use lib_models::error::ModelResult;
use serde::{Deserialize, Serialize};
use sqlb::{Fields, SqlxBindable};
use sqlx::{prelude::FromRow, PgPool};

use crate::routes::AppState;

use super::base::DbBmc;

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
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub deactivated_at: Option<NaiveDateTime>,
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
    created_at: &NaiveDateTime,
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
        .and_where("created_at", "<", NaiveDateTimeWrapper(created_at))
        .order_by("!created_at")
        .limit(3)
        .fetch_all::<_, ContentModel>(pool)
        .await?;
    Ok(row)
}

pub async fn get_ten_unseen_older<'q>(
    pool: &PgPool,
    created_at: &NaiveDateTime,
    username: &str,
) -> ModelResult<Vec<ContentModel>> {
    let rows = sqlx::query_as::<_, ContentModel>(&format!(
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

#[derive(Debug)]
pub struct NaiveDateTimeWrapper<'a>(&'a NaiveDateTime);

impl SqlxBindable for NaiveDateTimeWrapper<'_> {
    fn bind_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        query.bind(self.0)
    }
}

pub fn sort_by_predicted(
    posts: &mut Vec<ContentModel>,
    s: &AppState,
    num_taken: usize,
    user_id: i64,
) {
    let post_ids = posts.iter().map(|p| p.id).collect::<Vec<_>>();
    let predictions = s
        .ndarray_app_state
        .lock()
        .expect("err locking")
        .predict_all(user_id, &post_ids);

    let zipped = posts.iter().zip(predictions.iter()).collect::<Vec<_>>();
    *posts = zipped
        .iter()
        .sorted_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|&(x, _)| x.clone())
        .take(num_taken)
        .collect::<Vec<_>>();
}
