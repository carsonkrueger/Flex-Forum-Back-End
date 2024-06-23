use super::NestedRoute;
use super::RouterResult;
use crate::libs::ctx::Ctx;
use crate::middleware::auth_mw::AUTH_TOKEN;
use crate::models::base;
use crate::models::user_model;
use crate::models::user_model::UserModel;
use crate::services::auth::check_username;
use axum::extract::Path;
use axum::routing::delete;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use tower_cookies::Cookie;
use tower_cookies::Cookies;

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
    let users = user_model::list_by_username::<UserModel, ReadUserModel>(
        5,
        0,
        &username.to_lowercase(),
        &pool,
    )
    .await?;
    Ok(Json(users))
}

pub async fn delete_user(
    ctx: Ctx,
    cookies: Cookies,
    Path(username): Path<String>,
    State(pool): State<PgPool>,
) -> RouterResult<()> {
    check_username(&username, ctx.jwt())?;

    base::delete::<UserModel, &str>("username", &username, &pool).await?;
    cookies.remove(Cookie::from(AUTH_TOKEN));

    Ok(())
}
