use super::NestedRoute;
use super::{RouteError, RouterResult};
use crate::libs::hash_scheme::HashScheme;
use crate::libs::jwt::JWT;
use crate::libs::validation::{validate_struct, RE_NAME, RE_USERNAME};
use crate::middleware::auth_mw::{AUTH_TOKEN, JWT_SECRET};
use crate::models::user_model::{username_or_email_exists, CreateUserModel, UserModel};
use crate::models::{self};
use crate::services::hash_services::{self, verify};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

pub struct LoginSignupRoute;

impl NestedRoute<PgPool> for LoginSignupRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<PgPool> {
        Router::new()
            .route("/signup", post(sign_up))
            .route("/login", post(log_in))
    }
}

#[derive(Deserialize, Validate, Fields)]
pub struct SignUpModel {
    #[validate(length(min = 1, max = 32, message = "Invalid first name length"))]
    #[validate(regex(path = "*RE_NAME"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 32, message = "Invalid last name length"))]
    #[validate(regex(path = r#"*RE_NAME"#, message = "Invalid last name"))]
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
        return Err(RouteError::Validation(e.to_string()));
    }

    let taken_str = username_or_email_exists(&body.username, &body.email, &pool).await?;
    if let Some(taken) = taken_str {
        return Err(RouteError::AlreadyTaken(taken));
    }

    let hash_scheme = HashScheme::Argon2;
    let (pwd_hash, pwd_salt) = hash_services::hash(body.password.as_bytes(), &hash_scheme)?;

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
    cookies: Cookies,
    Json(body): Json<LoginModel>,
) -> RouterResult<Json<i64>> {
    validate_struct(&body)?;

    let option_hash =
        models::user_model::get_one_by_username::<UserModel, HashModel>(&body.username, &pool)
            .await?;

    let hash_model = option_hash.ok_or(RouteError::LoginFail)?;

    verify(
        body.password.as_bytes(),
        &hash_model.pwd_salt,
        &hash_model.pwd_hash,
    )?;

    let result_jwt = JWT::new(hash_model.id, &JWT_SECRET)?;

    let expires = tower_cookies::cookie::time::OffsetDateTime::now_utc()
        + tower_cookies::cookie::time::Duration::minutes(4);

    let mut auth_cookie = Cookie::new(AUTH_TOKEN, result_jwt.to_string());
    auth_cookie.set_expires(expires);
    auth_cookie.set_path("/");
    cookies.add(auth_cookie);

    Ok(Json(hash_model.id))
}
