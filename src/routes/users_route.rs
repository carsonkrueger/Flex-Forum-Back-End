use super::NestedRoute;
use super::RouterResult;
use crate::libs::ctx::Ctx;
use crate::middleware::auth_mw::AUTH_TOKEN;
use crate::models::base;
use crate::models::following_model::FollowingModel;
use crate::models::user_model;
use crate::models::user_model::UserModel;
use crate::services::auth::check_username;
use crate::AppState;
use axum::extract::Path;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};
use serde::Deserialize;
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
            .route("/follow/:following", post(follow_user))
            .route("/follow/:following", delete(unfollow_user))
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

#[derive(Deserialize, Fields)]
pub struct FollowingCreateModel {
    follower: String,
    following: String,
}

async fn follow_user(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(following): Path<String>,
) -> RouterResult<()> {
    let follow = FollowingCreateModel {
        follower: ctx.jwt().username().to_string(),
        following,
    };
    super::models::base::create::<FollowingModel, FollowingCreateModel>(follow, &s.pool).await?;
    Ok(())
}

async fn unfollow_user(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(following): Path<String>,
) -> RouterResult<()> {
    super::models::base::delete_with_both::<FollowingModel, _, _>(
        "follower",
        ctx.jwt().username(),
        "following",
        following,
        &s.pool,
    )
    .await?;
    Ok(())
}
