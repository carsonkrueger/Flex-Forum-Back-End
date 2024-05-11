use self::{
    hello_world::HelloWorldRoute, login_signup_route::LoginSignupRoute, users_route::UserRoute,
};
use crate::{
    middleware::{
        auth_mw::{ctx_resolver, validate_auth},
        logger_mw::logger,
    },
    models,
};
use axum::{
    body::Body,
    http::StatusCode,
    middleware::{from_fn, map_response},
    response::IntoResponse,
    Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_cookies::CookieManagerLayer;

mod hello_world;
mod login_signup_route;
mod users_route;

pub type RouterResult<T> = std::result::Result<T, RouteError>;

#[derive(Clone, Serialize, Debug)]
pub enum RouteError {
    MissingAuthCookie,
    LoginFail,
    InvalidAuth,
    Validation(String),
    AlreadyTaken(String),
    HashError,
    // Used to hide error from users
    Unknown,
}

pub trait NestedRoute<S> {
    const PATH: &'static str;
    fn router() -> Router<S>;
}

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .nest(HelloWorldRoute::PATH, HelloWorldRoute::router())
        .nest(UserRoute::PATH, UserRoute::router())
        .layer(from_fn(validate_auth))
        .nest(LoginSignupRoute::PATH, LoginSignupRoute::router())
        .layer(from_fn(ctx_resolver))
        .layer(map_response(logger))
        .layer(CookieManagerLayer::new())
        .with_state(pool)
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response<Body> {
        let mut response = StatusCode::from(&self).into_response();
        let body = Body::new(self.to_string());
        let _ = std::mem::replace(response.body_mut(), body);
        response
    }
}

impl From<&RouteError> for StatusCode {
    fn from(value: &RouteError) -> Self {
        use RouteError::*;
        match value {
            InvalidAuth => StatusCode::FORBIDDEN,
            MissingAuthCookie => StatusCode::FORBIDDEN,
            LoginFail => StatusCode::UNAUTHORIZED,
            AlreadyTaken(..) => StatusCode::CONFLICT,
            Validation(..) => StatusCode::BAD_REQUEST,
            HashError | Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<crate::models::ModelError> for RouteError {
    fn from(_value: models::ModelError) -> Self {
        RouteError::Unknown
    }
}

impl From<hash_lib::error::HashError> for RouteError {
    fn from(_value: hash_lib::error::HashError) -> Self {
        RouteError::HashError
    }
}

impl From<argon2::password_hash::Error> for RouteError {
    fn from(_value: argon2::password_hash::Error) -> Self {
        RouteError::HashError
    }
}

impl ToString for RouteError {
    fn to_string(&self) -> String {
        use RouteError::*;
        match &self {
            AlreadyTaken(s) => format!("{} already taken", s),
            InvalidAuth => format!("Invalid auth token"),
            LoginFail => format!("Login failed"),
            MissingAuthCookie => format!("Missing auth token"),
            Validation(s) => s.to_string(),
            HashError | Unknown => format!("Unknown error"),
        }
    }
}
