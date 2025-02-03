use crate::middleware::auth_mw::{AUTH_TOKEN, JWT_SECRET};
use crate::model::base;
use crate::model::schemas::user_management::users::{Users, UsersIden};
use crate::route::error::{RouteError, RouteResult};
use crate::route::NestedRoute;
use crate::services::users::{create_user, verify_user};
use crate::util::jwt::JWT;
use crate::util::validation::{validate_struct, RE_NAME, RE_USERNAME};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
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

#[derive(Deserialize, Validate, FromRow)]
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
    cookies: Cookies,
    Json(mut body): Json<SignUpModel>,
) -> RouteResult<StatusCode> {
    if let Err(e) = body.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }

    body.username = body.username.trim().to_lowercase();
    body.password = body.password.trim().to_string();

    // let hasher = Argon2V01;
    // let (pwd_hash, pwd_salt) = hasher.hash(&body.password)?;

    let user = create_user(&body.username, &body.email, &body.password, &s.pool).await?;
    let jwt = JWT::new(user.id, user.username, Vec::new());
    let jwt_str = jwt.encode(JWT_SECRET.as_bytes())?;
    let cookie = Cookie::new(AUTH_TOKEN, jwt_str);
    cookies.add(cookie);

    s.ndarray_app_state
        .lock()
        .expect("err locking")
        .add_user(user.id)
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

/// logs user in with username & password
pub async fn log_in(
    State(s): State<AppState>,
    cookies: Cookies,
    Json(mut body): Json<LoginModel>,
) -> RouteResult<()> {
    validate_struct(&body)?;

    body.username = body.username.trim().to_lowercase();
    body.password = body.password.trim().to_string();

    let user = base::get_one_with::<Users, Users>(UsersIden::Username, body.username, &s.pool)
        .await?
        .ok_or(RouteError::InvalidAuth)?;
    verify_user(&user, &body.password)?;
    let jwt = JWT::new(user.id, user.username, Vec::new());
    let jwt_str = jwt.encode(JWT_SECRET.as_bytes())?;
    let cookie = Cookie::new(AUTH_TOKEN, jwt_str);

    // let expires = tower_cookies::cookie::time::OffsetDateTime::now_utc()
    //     + tower_cookies::cookie::time::Duration::minutes(JWT_LIFE_IN_MINUTES);
    // auth_cookie.set_expires(expires);
    // auth_cookie.set_path("/");
    cookies.add(cookie);

    Ok(())
}
