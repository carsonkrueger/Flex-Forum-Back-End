use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use super::ModelResult;

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct InteractionsMatrixModel {
    pub user_id: i64,
    pub post_id: i64,
    pub is_liked: i32,
    pub is_following: i32,
    pub username: String,
}

pub async fn build_model(pool: &PgPool) -> ModelResult<Vec<InteractionsMatrixModel>> {
    let join_query = sqlx::query_as::<_, InteractionsMatrixModel>(
        "
        SELECT
            u.username,
            u.id as user_id,
            p.id as post_id,
            COALESCE(like_data.is_liked, 0) as is_liked,
            COALESCE(follow_data.is_following, 0) as is_following
        FROM user_management.users u
        CROSS JOIN post_management.posts p
        LEFT JOIN
            (
                SELECT
                    username,
                    post_id,
                    1 as is_liked
                FROM
                    post_management.likes
            ) like_data
            ON u.username = like_data.username AND p.id = like_data.post_id
        LEFT JOIN
            (
                SELECT
                    follower,
                    following,
                    1 as is_following
                FROM
                    user_management.following
            ) follow_data
            ON u.username = follow_data.follower AND p.username = follow_data.following
        ;
            ",
    )
    .fetch_all(pool)
    .await?;

    Ok(join_query)
}
