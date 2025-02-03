use crate::model::schema::Schema;
use lib_macros::{iterator_def, iterator_iden_def, schema_table_def};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow)]
#[enum_def]
#[schema_table_def(Schema::UserManagement, ProfilePictureIden::Table)]
#[iterator_iden_def(ProfilePictureIden)]
#[iterator_def]
pub struct ProfilePicture {
    pub id: i64,
    pub username: String,
}
