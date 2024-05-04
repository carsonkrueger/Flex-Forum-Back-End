use crate::lib::hash_scheme::HashScheme;
use crate::models::user_models::UserModel;
use crate::view_models::user_view_models::{CreateUserViewModel, ReadUserViewModel};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDateTime;
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;
use validator::Validate;

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserViewModel>,
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
        return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
    }

    (StatusCode::CREATED, "".to_owned())
}

pub async fn get_user(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let result = sqlx::query_as!(
        UserModel,
        r#"
        SELECT 
            id,
            first_name,
            last_name,
            email,
            username,
            password_hash,
            hash_scheme as "hash_scheme: HashScheme",
            created_at as "created_at: _",
            deactivated_at as "deactivated_at: _",
            salt
        FROM user_management.users WHERE id = $1;"#,
        id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(um) => Ok((StatusCode::ACCEPTED, Json(ReadUserViewModel::from(um)))),
    }
}
