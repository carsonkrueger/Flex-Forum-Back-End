use axum::{extract::State, routing::get, Json, Router};
use lib_macros::iterator_column_def;
use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::{
    model::{
        base,
        schemas::workout_management::exercise_presets::{ExercisePreset, ExercisePresetIden},
    },
    route::{error::RouteResult, AppState, NestedRoute},
};

pub struct ExercisePresetRoute;

impl NestedRoute<AppState> for ExercisePresetRoute {
    const PATH: &'static str = "/exercise-presets";
    fn router() -> Router<AppState> {
        Router::new().route("/", get(get_presets))
    }
}

#[derive(FromRow, Serialize)]
#[iterator_column_def(ExercisePresetIden)]
pub struct ReadExercisePresetModel {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

pub async fn get_presets(
    State(s): State<AppState>,
) -> RouteResult<Json<Vec<ReadExercisePresetModel>>> {
    let res = base::get_all::<ExercisePreset, ReadExercisePresetModel>(&s.pool).await?;
    Ok(Json(res))
}
