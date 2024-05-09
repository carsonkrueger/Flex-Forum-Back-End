use self::{hello_world::HelloWorldRoute, users::UserRoute};
use crate::{
    middleware::auth::{ctx_resolver, validate_auth},
    models,
};
use axum::{body::Body, http::StatusCode, middleware::from_fn, response::IntoResponse, Router};
use serde::Serialize;
use sqlx::PgPool;

mod hello_world;
mod users;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Serialize)]
pub enum Error {
    MissingAuthCookie,
    LoginFail,
    InvalidAuth,
    Validation(String),
    AlreadyTaken(String),
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
        .layer(from_fn(validate_auth))
        .layer(from_fn(ctx_resolver))
        .nest(UserRoute::PATH, UserRoute::router())
        .with_state(pool)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response<Body> {
        let mut response = StatusCode::from(&self).into_response();
        let body = Body::new(self.to_string());
        let _ = std::mem::replace(response.body_mut(), body);
        response
    }
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> Self {
        use Error::*;
        match value {
            InvalidAuth => StatusCode::FORBIDDEN,
            MissingAuthCookie => StatusCode::FORBIDDEN,
            LoginFail => StatusCode::UNAUTHORIZED,
            AlreadyTaken(..) => StatusCode::CONFLICT,
            Validation(..) => StatusCode::BAD_REQUEST,
            Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<crate::models::Error> for Error {
    fn from(value: models::Error) -> Self {
        match value {
            models::Error::Sqlx(_) => Error::Unknown,
        }
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        use Error::*;
        match &self {
            AlreadyTaken(s) => format!("{} already taken", s),
            InvalidAuth => format!("Invalid auth token"),
            LoginFail => format!("Login failed"),
            MissingAuthCookie => format!("Missing auth token"),
            Validation(s) => s.to_string(),
            Unknown => format!("Unknown error"),
        }
    }
}
