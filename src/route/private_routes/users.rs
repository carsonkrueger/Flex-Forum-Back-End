use crate::middleware::auth_mw::AUTH_TOKEN;
use crate::model::base::BaseModel;
use crate::model::base::BaseModelTrait;
use crate::model::schemas::post_management::following::Following;
use crate::model::schemas::post_management::following::FollowingIden;
use crate::model::schemas::user_management::users::ReadUserModel;
use crate::model::schemas::user_management::users::Users;
use crate::model::schemas::user_management::users::UsersIden;
use crate::model::schemas::user_management::users::UsersModelTrait;
use crate::route::error::RouteResult;
use crate::route::NestedRoute;
use crate::util::ctx::Ctx;
use crate::AppState;
use axum::extract::Path;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};
use lib_macros::iterator_def;
use lib_macros::iterator_iden_def;
use serde::Deserialize;
use tower_cookies::Cookie;
use tower_cookies::Cookies;

pub struct UserRoute;

impl NestedRoute<AppState> for UserRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<AppState> {
        Router::new()
            .route("/:username", get(get_user))
            .route("/list/:username", get(list_users))
            .route("/delete", delete(delete_user))
            .route("/follow/:following", post(follow_user))
            .route("/follow/:following", delete(unfollow_user))
    }
}

pub async fn get_user(
    _ctx: Ctx,
    Path(username): Path<String>,
    State(s): State<AppState>,
) -> RouteResult<Json<Option<ReadUserModel>>> {
    let read_user = BaseModel::get_one_with::<Users, ReadUserModel>(
        UsersIden::Username,
        &username,
        &mut *s.pool.acquire().await?,
    )
    .await?;
    Ok(Json(read_user))
}

pub async fn list_users(
    _ctx: Ctx,
    Path(username): Path<String>,
    State(s): State<AppState>,
) -> RouteResult<Json<Vec<ReadUserModel>>> {
    let users = Users::list_by_username::<BaseModel>(
        5,
        0,
        &username.to_lowercase(),
        &mut *s.pool.acquire().await?,
    )
    .await?;
    Ok(Json(users))
}

pub async fn delete_user(ctx: Ctx, cookies: Cookies, State(s): State<AppState>) -> RouteResult<()> {
    BaseModel::delete_one_with::<Users>(
        UsersIden::Username,
        ctx.jwt().username().into(),
        &mut *s.pool.acquire().await?,
    )
    .await?;
    cookies.remove(Cookie::from(AUTH_TOKEN));

    Ok(())
}

#[derive(Deserialize)]
#[iterator_iden_def(FollowingIden)]
#[iterator_def(FollowingIden)]
pub struct FollowingCreateModel {
    follower: String,
    following: String,
}

async fn follow_user(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(following): Path<String>,
) -> RouteResult<()> {
    let follow = FollowingCreateModel {
        follower: ctx.jwt().username().to_string(),
        following,
    };
    BaseModel::insert_returning::<Following, FollowingCreateModel>(
        follow,
        &mut *s.pool.acquire().await?,
    )
    .await?;
    Ok(())
}

async fn unfollow_user(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(following): Path<String>,
) -> RouteResult<()> {
    BaseModel::delete_one_with_both::<Following>(
        FollowingIden::Follower,
        ctx.jwt().username().into(),
        FollowingIden::Following,
        following.into(),
        &mut *s.pool.acquire().await?,
    )
    .await?;
    Ok(())
}
