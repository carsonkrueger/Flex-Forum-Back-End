use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::base::DbBmc;

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
pub struct ExercisePresetModel {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

impl DbBmc for ExercisePresetModel {
    const TABLE: &'static str = "workout_management.exercise_presets";
}
