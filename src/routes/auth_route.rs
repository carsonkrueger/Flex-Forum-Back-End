use crate::libs::validation::{validate_struct, RE_NAME, RE_USERNAME};
use crate::middleware::auth_mw::{AUTH_TOKEN, JWT_SECRET};
use crate::models::user_model::{username_or_email_exists, CreateUserModel, UserModel};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use jwt::{JWT, JWT_LIFE_IN_MINUTES};
use lib_hash::hash_scheme::{HashScheme, Hasher};
use lib_hash::hashers::argon2_v01::Argon2V01;
use lib_routes::error::{RouteError, RouterResult};
use lib_routes::nested_route::NestedRoute;
use serde::Deserialize;
use sqlb::Fields;
use sqlx::prelude::FromRow;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

pub struct AuthRoute;

impl NestedRoute<AppState> for AuthRoute {
    const PATH: &'static str = "/users";
    fn router() -> Router<AppState> {
        Router::new()
            .route("/signup", post(sign_up))
            .route("/login", post(log_in))
    }
}

#[derive(Deserialize, Validate, Fields)]
pub struct SignUpModel {
    #[validate(length(min = 1, max = 32, message = "Invalid username length"))]
    #[validate(regex(path = "*RE_USERNAME", message = "Invalid username"))]
    pub username: String,
    #[validate(
        email(message = "Invalid email"),
        length(min = 1, max = 255, message = "Invalid email length")
    )]
    pub email: String,
    #[validate(length(min = 1, max = 32, message = "Invalid first name length"))]
    #[validate(regex(path = "*RE_NAME"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 32, message = "Invalid last name length"))]
    #[validate(regex(path = r#"*RE_NAME"#, message = "Invalid last name"))]
    pub last_name: String,
    #[validate(length(min = 1, max = 64, message = "Invalid password length"))]
    pub password: String,
}

pub async fn sign_up(
    State(s): State<AppState>,
    Json(mut body): Json<SignUpModel>,
) -> RouterResult<StatusCode> {
    if let Err(e) = body.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }

    body.username = body.username.trim().to_lowercase();
    body.password = body.password.trim().to_string();

    let taken_str = username_or_email_exists(&body.username, &body.email, &s.pool).await?;
    if let Some(taken) = taken_str {
        return Err(RouteError::AlreadyTaken(taken));
    }

    let hasher = Argon2V01;
    let (pwd_hash, pwd_salt) = hasher.hash(&body.password)?;

    let create_model = CreateUserModel {
        username: body.username,
        email: body.email,
        first_name: body.first_name,
        last_name: body.last_name,
        pwd_hash,
        pwd_salt: pwd_salt.to_string(),
        hash_scheme: hasher.into(),
    };

    let user_id = super::models::base::create::<UserModel, _>(create_model, &s.pool).await?;
    s.ndarray_app_state
        .lock()
        .expect("err locking")
        .add_user(user_id)
        .expect("err adding user");

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Validate)]
pub struct LoginModel {
    #[validate(length(min = 5, max = 32, message = "Invalid username length"))]
    #[validate(regex(path = "*RE_USERNAME"))]
    pub username: String,
    #[validate(length(min = 8, max = 32, message = "Invalid password length"))]
    pub password: String,
}

#[derive(FromRow, Fields, Debug)]
pub struct HashModel {
    username: String,
    hash_scheme: HashScheme,
    pwd_hash: String,
    pwd_salt: String,
}

/// logs user in with username & password
pub async fn log_in(
    State(s): State<AppState>,
    cookies: Cookies,
    Json(mut body): Json<LoginModel>,
) -> RouterResult<()> {
    validate_struct(&body)?;

    body.username = body.username.trim().to_lowercase();
    body.password = body.password.trim().to_string();

    let option_hash = super::models::base::get_one::<UserModel, HashModel, _>(
        "username",
        &body.username,
        &s.pool,
    )
    .await?;

    let hash_model = option_hash.ok_or(RouteError::LoginFail)?;

    let hasher = hash_model.hash_scheme.hasher();
    hasher.verify(&body.password, &hash_model.pwd_salt, &hash_model.pwd_hash)?;

    let result_jwt = JWT::new(hash_model.username.clone(), &JWT_SECRET)?;

    let mut auth_cookie = Cookie::new(AUTH_TOKEN, result_jwt.to_string());
    let expires = tower_cookies::cookie::time::OffsetDateTime::now_utc()
        + tower_cookies::cookie::time::Duration::minutes(JWT_LIFE_IN_MINUTES);
    auth_cookie.set_expires(expires);
    auth_cookie.set_path("/");
    cookies.add(auth_cookie);

    Ok(())
}
