use super::NestedRoute;
use super::RouterResult;
use crate::libs::ctx::Ctx;
use crate::middleware::auth_mw::AUTH_TOKEN;
use crate::models::base;
use crate::models::user_model;
use crate::models::user_model::UserModel;
use crate::services::auth::check_username;
use crate::AppState;
use axum::extract::Path;
use axum::routing::delete;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use tower_cookies::Cookie;
use tower_cookies::Cookies;

pub struct UserRoute;

impl NestedRoute<AppState> for UserRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<AppState> {
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
    State(s): State<AppState>,
) -> RouterResult<Json<Option<ReadUserModel>>> {
    let read_user =
        super::models::base::get_one::<UserModel, _, _>("username", &username, &s.pool).await?;
    Ok(Json(read_user))
}

pub async fn list_users(
    _ctx: Ctx,
    Path(username): Path<String>,
    State(s): State<AppState>,
) -> RouterResult<Json<Vec<ReadUserModel>>> {
    let users = user_model::list_by_username::<UserModel, ReadUserModel>(
        5,
        0,
        &username.to_lowercase(),
        &s.pool,
    )
    .await?;
    Ok(Json(users))
}

pub async fn delete_user(
    ctx: Ctx,
    cookies: Cookies,
    Path(username): Path<String>,
    State(s): State<AppState>,
) -> RouterResult<()> {
    check_username(&username, ctx.jwt())?;

    base::delete::<UserModel, &str>("username", &username, &s.pool).await?;
    cookies.remove(Cookie::from(AUTH_TOKEN));

    Ok(())
}
