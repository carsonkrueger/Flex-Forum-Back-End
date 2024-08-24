use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct LikesModel {
    pub post_id: i64,
    pub username: String,
    pub is_liked: bool,
    pub is_following: bool,
    pub num_comments: u32,
}
