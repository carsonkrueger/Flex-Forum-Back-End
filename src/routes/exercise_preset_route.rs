use axum::{extract::State, routing::get, Json, Router};
use lib_routes::error::RouterResult;
use serde::Serialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;

use crate::models::exercise_preset_model::ExercisePresetModel;

use lib_routes::nested_route::NestedRoute;

use super::AppState;

pub struct ExercisePresetRoute;

impl NestedRoute<AppState> for ExercisePresetRoute {
    const PATH: &'static str = "/exercise-presets";
    fn router() -> Router<AppState> {
        Router::new().route("/", get(get_presets))
    }
}

#[derive(Fields, FromRow, Serialize)]
pub struct ReadExercisePresetModel {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

pub async fn get_presets(
    State(s): State<AppState>,
) -> RouterResult<Json<Vec<ReadExercisePresetModel>>> {
    let res = super::models::base::get_all::<ExercisePresetModel, _>(&s.pool, Some("id")).await?;
    Ok(Json(res))
}
