use self::{auth_route::LoginSignupRoute, hello_world::HelloWorldRoute, users_route::UserRoute};
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
use content_route::ContentRoute;
use sqlx::PgPool;
use tower_cookies::CookieManagerLayer;

mod auth_route;
mod content_route;
mod hello_world;
mod users_route;

pub type RouterResult<T> = std::result::Result<T, RouteError>;

#[derive(Debug, Clone)]
pub enum RouteError {
    Unauthorized,
    MissingAuthCookie,
    MissingJWTSignature,
    LoginFail,
    InvalidAuth,
    Validation(String),
    AlreadyTaken(String),
    HashError,
    ExpiredAuthToken,
    ChronoParseError,
    InvalidContentType(String),
    IOError(String),
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
        .nest(ContentRoute::PATH, ContentRoute::router())
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
            ExpiredAuthToken | MissingJWTSignature | InvalidAuth | MissingAuthCookie
            | LoginFail | Unauthorized => StatusCode::UNAUTHORIZED,
            AlreadyTaken(..) => StatusCode::CONFLICT,
            Validation(..) | InvalidContentType(..) => StatusCode::BAD_REQUEST,
            IOError(..) | HashError | ChronoParseError | Unknown => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl From<crate::models::ModelError> for RouteError {
    fn from(_value: models::ModelError) -> Self {
        RouteError::Unknown
    }
}

impl From<hash_lib::error::HashError> for RouteError {
    fn from(value: hash_lib::error::HashError) -> Self {
        match value {
            hash_lib::error::HashError::VerificationFail => RouteError::InvalidAuth,
            _ => RouteError::HashError,
        }
    }
}

impl From<argon2::password_hash::Error> for RouteError {
    fn from(_value: argon2::password_hash::Error) -> Self {
        RouteError::HashError
    }
}

impl From<chrono::ParseError> for RouteError {
    fn from(_value: chrono::ParseError) -> Self {
        Self::ChronoParseError
    }
}

impl From<std::io::Error> for RouteError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.to_string())
    }
}

impl ToString for RouteError {
    fn to_string(&self) -> String {
        use RouteError::*;
        match &self {
            AlreadyTaken(s) => format!("{} already taken", s),
            ExpiredAuthToken => format!("Auth token expired"),
            InvalidAuth => format!("Invalid auth token"),
            LoginFail => format!("Login failed"),
            MissingAuthCookie => format!("Missing auth token"),
            MissingJWTSignature => format!("Missing JWT signature"),
            Validation(s) => s.to_string(),
            IOError(..) | HashError | ChronoParseError | Unknown => format!("Unknown error"),
            InvalidContentType(f) => format!("{}: Invalid content type", f),
            Unauthorized => "".to_string(),
        }
    }
}
