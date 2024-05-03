use crate::lib::hash_scheme::HashScheme;
use crate::lib::validation::RE_NAME;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, PgPool};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct UserViewModel {
    #[validate(length(min = 1, max = 32, message = "Invalid first name length"))]
    #[validate(regex(path = "*RE_NAME"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 32, message = "Invalid last name length"))]
    #[validate(regex(path = "*RE_NAME", message = "Invalid last name"))]
    pub last_name: String,
    #[validate(
        email(message = "Invalid email", code = "a code"),
        length(min = 1, max = 255, message = "Invalid email length")
    )]
    pub email: String,
    #[validate(length(min = 1, max = 32, message = "Invalid username"))]
    pub username: String,
    #[validate(length(min = 1, max = 64, message = "Invalid password"))]
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserModel {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    username: String,
    password: String,
    salt: String,
    hash_scheme: HashScheme,
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<UserViewModel>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return (StatusCode::BAD_REQUEST, e.to_string());
    }

    let result: Result<PgQueryResult, sqlx::Error> = sqlx::query!(
        "
    INSERT INTO user_management.users (
        first_name,
        last_name,
        email,
        username,
        password_hash,
        salt,
        hash_scheme
    ) VALUES ($1, $2, $3, $4, $5, $6, $7);
    ",
        body.first_name,
        body.last_name,
        body.email,
        body.username,
        body.password,
        "salt",
        HashScheme::Argon2 as HashScheme
    )
    .execute(&pool)
    .await;

    if let Err(e) = result {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    (StatusCode::CREATED, "".to_owned())
}
