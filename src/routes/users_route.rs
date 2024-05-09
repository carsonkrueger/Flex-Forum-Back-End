use super::NestedRoute;
use super::{Error, Result};
use crate::libs::jwt::JWT;
use crate::libs::validation::{validate_struct, RE_NAME, RE_USERNAME};
use crate::middleware::auth::AUTH_TOKEN;
use crate::models;
use crate::models::user_model::{
    username_or_email_exists, CreateUserModel, ReadUserModel, UserModel,
};
use crate::services::hash_services::{self, verify};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::Router;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

pub struct UserRoute;

impl NestedRoute<PgPool> for UserRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<PgPool> {
        Router::new()
            .route("/", post(sign_up))
            .route("/:id", get(get_user))
            .route("/login", post(log_in))
    }
}

#[derive(Deserialize, Validate, Fields)]
pub struct SignUpModel {
    #[validate(length(min = 1, max = 32, message = "Invalid first name length"))]
    #[validate(regex(path = "*RE_NAME"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 32, message = "Invalid last name length"))]
    #[validate(regex(path = "*RE_NAME", message = "Invalid last name"))]
    pub last_name: String,
    #[validate(
        email(message = "Invalid email"),
        length(min = 1, max = 255, message = "Invalid email length")
    )]
    pub email: String,
    #[validate(length(min = 1, max = 32, message = "Invalid username length"))]
    #[validate(regex(path = "*RE_USERNAME", message = "Invalid username"))]
    pub username: String,
    #[validate(length(min = 1, max = 64, message = "Invalid password length"))]
    pub password: String,
}

pub async fn sign_up(
    State(pool): State<PgPool>,
    Json(body): Json<SignUpModel>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return Err(Error::Validation(e.to_string()));
    }

    let taken_str = username_or_email_exists(&body.username, &body.email, &pool).await?;
    if let Some(taken) = taken_str {
        return Err(Error::AlreadyTaken(taken));
    }

    let (pwd_hash, pwd_salt, hash_scheme) = hash_services::hash(body.password.as_bytes())?;

    let create_model = CreateUserModel {
        username: body.username,
        email: body.email,
        first_name: body.first_name,
        last_name: body.last_name,
        pwd_hash,
        pwd_salt: pwd_salt.to_string(),
        hash_scheme,
    };

    let id = models::user_model::create(&pool, create_model).await?;

    Ok((StatusCode::CREATED, Json(id)))
}

pub async fn get_user(Path(id): Path<i64>, State(pool): State<PgPool>) -> impl IntoResponse {
    let user_result = sqlx::query_as::<_, ReadUserModel>(
        "
        SELECT
            first_name,
            last_name,
            username
        FROM user_management.users WHERE id = $1;
        ",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await;

    match user_result {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Ok(Some(user)) => Ok((StatusCode::ACCEPTED, Json(user))),
    }
}

#[derive(Deserialize, Validate)]
pub struct LoginModel {
    #[validate(length(min = 1, max = 32, message = "Invalid username length"))]
    #[validate(regex(path = "*RE_USERNAME"))]
    pub username: String,
    #[validate(length(min = 1, max = 32, message = "Invalid password length"))]
    pub password: String,
}

#[derive(FromRow, Fields)]
pub struct HashModel {
    id: i64,
    pwd_hash: String,
    pwd_salt: String,
}

/// logs user in with username & password
pub async fn log_in(
    State(pool): State<PgPool>,
    // cookies: Cookies,
    Json(body): Json<LoginModel>,
) -> Result<Json<i64>> {
    validate_struct(&body)?;

    let option_hash =
        models::user_model::get_one_by_username::<UserModel, HashModel>(&body.username, &pool)
            .await?;

    let hash_model = option_hash.ok_or(Error::LoginFail)?;
    let verified = verify(
        body.password.as_bytes(),
        &hash_model.pwd_salt,
        &hash_model.pwd_hash,
    )?;

    if !verified {
        return Err(Error::LoginFail);
    }

    // let jwt = JWT::
    // let cookie = Cookie::new(AUTH_TOKEN, "");
    // cookies.add(cookie);

    Ok(Json(hash_model.id))
}
