use lib_models::error::ModelResult;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::{prelude::FromRow, Pool, Postgres};

use super::base::{self, DbBmc};

#[derive(Deserialize, Serialize, FromRow, Debug, Clone, Fields)]
pub struct FollowingModel {
    pub id: i64,
    pub follower: String,
    pub following: String,
}

impl DbBmc for FollowingModel {
    const TABLE: &'static str = "user_management.following";
}

pub async fn is_following(
    pool: &Pool<Postgres>,
    follower: &str,
    following: &str,
) -> ModelResult<bool> {
    let res = base::get_one_with_both::<FollowingModel, FollowingModel, &str, &str>(
        "follower",
        follower,
        "following",
        following,
        pool,
    )
    .await?;
    return Ok(res.is_some());
}
