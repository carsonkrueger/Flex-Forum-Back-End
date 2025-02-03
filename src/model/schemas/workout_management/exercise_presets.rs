use crate::model::schema::Schema;
use lib_macros::{iterator_column_def, schema_table_def};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
#[enum_def]
#[schema_table_def(Schema::WorkoutManagement, ExercisePresetIden::Table)]
#[iterator_column_def(ExercisePresetIden)]
pub struct ExercisePreset {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}
