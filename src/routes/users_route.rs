use super::NestedRoute;
use super::RouterResult;
use crate::libs::ctx::Ctx;
use crate::models::base;
use crate::models::user_model;
use crate::models::user_model::UserModel;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use sqlx::PgPool;

pub struct UserRoute;

impl NestedRoute<PgPool> for UserRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<PgPool> {
        Router::new()
            .route("/:username", get(get_user))
            .route("/list/:username", get(list_users))
            .route("/delete/:id", delete(delete_user))
    }
}

#[derive(Fields, FromRow, Serialize)]
pub struct ReadUserModel {
    username: String,
    first_name: String,
    last_name: String,
}

pub async fn get_user(
    _ctx: Ctx,
    Path(username): Path<String>,
    State(pool): State<PgPool>,
) -> RouterResult<Json<Option<ReadUserModel>>> {
    let read_user =
        user_model::get_one_by_username::<UserModel, ReadUserModel>(&username, &pool).await?;
    Ok(Json(read_user))
}

pub async fn list_users(
    _ctx: Ctx,
    Path(username): Path<String>,
    State(pool): State<PgPool>,
) -> RouterResult<Json<Vec<ReadUserModel>>> {
    let users =
        user_model::list_by_username::<UserModel, ReadUserModel>(2, 0, &username, &pool).await?;
    Ok(Json(users))
}

pub async fn delete_user(
    ctx: Ctx,
    Path(id): Path<i64>,
    State(pool): State<PgPool>,
) -> RouterResult<StatusCode> {
    if ctx.jwt().id() != &id {
        return Ok(StatusCode::FORBIDDEN);
    }

    base::delete::<UserModel>(id, &pool).await?;

    Ok(StatusCode::ACCEPTED)
}
