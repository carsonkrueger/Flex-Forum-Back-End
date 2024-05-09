use super::Error;
use super::NestedRoute;
use crate::libs::validation::validate;
use crate::libs::validation::{RE_NAME, RE_USERNAME};
use crate::models;
use crate::models::base;
use crate::models::user::{username_or_email_exists, CreateUserModel, ReadUserModel};
use crate::services::hash_services::{self, verify};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::Router;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use sqlb::Fields;
use sqlx::PgPool;
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

    let id = models::user::create(&pool, create_model).await?;

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

/// logs user in with username & password
pub async fn log_in(State(pool): State<PgPool>, Json(body): Json<LoginModel>) -> impl IntoResponse {
    validate(body)?;

    // let query_result = sqlx::query_scalar::<_, (i64, String, String)>(
    //     "SELECT (id, pwd_hash, pwd_salt) FROM user_management.users WHERE email = $1;",
    // )
    // .bind(body.username)
    // .fetch_optional(&pool)
    // .await;

    let (verify_result, id) = match query_result {
        Ok(Some(ref row)) => (
            verify(body.password.as_bytes(), &row.2, &row.1),
            row.0.to_string(),
        ),
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match verify_result {
        Ok(true) => Ok((StatusCode::ACCEPTED, id)),
        Ok(false) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
