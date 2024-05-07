use super::NestedRoute;
use crate::lib::hash_scheme::HashScheme;
use crate::services::hash_services::{self, verify};
use crate::services::user_services::email_exists;
use crate::view_models::login_view_models::LoginModel;
use crate::view_models::user_view_models::{CreateUserViewModel, ReadUserViewModel};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::Router;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub struct UserRoute;

impl NestedRoute<PgPool> for UserRoute {
    fn path<'a>() -> &'a str {
        "/users"
    }
    fn router() -> Router<PgPool> {
        Router::new()
            .route("/", post(create_user))
            .route("/:id", get(get_user))
            .route("/login", post(login))
    }
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserViewModel>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return Ok((StatusCode::BAD_REQUEST, e.to_string()));
    }

    let exists_result = email_exists(&body.email, &pool).await;
    match exists_result {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(true) => return Err(StatusCode::CONFLICT),
        Ok(false) => (),
    }

    let (hash, salt) = hash_services::hash(body.password.as_bytes())?;
    let query_result = sqlx::query_scalar::<_, Uuid>(
        "
            INSERT INTO user_management.users (
                first_name,
                last_name,
                email,
                username,
                password_hash,
                salt,
                hash_scheme
            ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING
                id,
                first_name,
                last_name,
                email,
                username;
        ",
    )
    .bind(body.first_name)
    .bind(body.last_name)
    .bind(body.email)
    .bind(body.username)
    .bind(hash.clone())
    .bind(salt.to_string())
    .bind(HashScheme::Argon2)
    .fetch_one(&pool)
    .await;

    match query_result {
        Ok(id) => Ok((StatusCode::CREATED, id.to_string())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_user(Path(id): Path<Uuid>, State(pool): State<PgPool>) -> impl IntoResponse {
    let user_result = sqlx::query_as::<_, ReadUserViewModel>(
        "
        SELECT
            id,
            first_name,
            last_name,
            email,
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

pub async fn login(State(pool): State<PgPool>, Json(body): Json<LoginModel>) -> impl IntoResponse {
    let query_result = sqlx::query_scalar::<_, (Uuid, String, String)>(
        "SELECT (id, password_hash, salt) FROM user_management.users WHERE email = $1;",
    )
    .bind(body.email)
    .fetch_optional(&pool)
    .await;

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
