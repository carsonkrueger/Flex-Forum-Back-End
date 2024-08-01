use super::base::DbBmc;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow, Fields)]
pub struct ProfilePictureModel {
    pub id: i64,
    pub username: String,
}

impl DbBmc for ProfilePictureModel {
    const TABLE: &'static str = "user_management.profile_pictures";
}
