use lib_macros::{iterator_iden_def, schema_table_def};
use sea_query::{enum_def, IntoTableRef};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::model::{error::ModelResult, schema::Schema};

#[derive(Deserialize, Serialize, FromRow, Debug)]
#[enum_def]
#[schema_table_def(Schema::PostManagement, SeenPostsIden::Table)]
#[iterator_iden_def(SeenPostsIden)]
pub struct SeenPosts {
    pub id: i64,
    pub post_id: i64,
    pub username: String,
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct SeenPostCreateModel {
    pub post_id: i64,
    pub username: String,
}

pub async fn seen(pool: &PgPool, username: &str, post_id: i64) -> ModelResult<()> {
    // let e = base::create::<SeenPostsModel, SeenPostCreateModel>(seen, pool).await;
    let _id = sqlx::query_scalar::<_, i64>(&format!(
        "INSERT INTO {:?} (post_id, username) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING id;",
        SeenPostsIden::Table.into_table_ref()
    ))
    .bind(post_id)
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(())
}
