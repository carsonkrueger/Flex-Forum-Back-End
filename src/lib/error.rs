use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Serialize)]
pub enum Error {
    MissingAuthCookie,
    LoginFail,
    InvalidAuth,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let mut response = StatusCode::from(&self).into_response();
        response.extensions_mut().insert(self); // insert error enum into response
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
        }
    }
}
