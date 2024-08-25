use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct InteractionsMatrixModel {
    pub user_id: i64,
    pub post_id: i64,
    pub username: String,
}
