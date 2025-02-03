use aws_sdk_s3::{
    error::SdkError,
    operation::{
        delete_object::DeleteObjectError, get_object::GetObjectError, put_object::PutObjectError,
    },
};
use axum::{body::Body, http::StatusCode, response::IntoResponse};
use lib_multipart::error::LibMultipartError;

use crate::model::error::ModelError;

pub type RouteResult<T> = std::result::Result<T, RouteError>;

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
    LibMultipartError(LibMultipartError),
    IOError(String),
    Sqlx(String),
    AwsSdkError(String),
    JWTError(String),
    // Used to hide error from users
    Unknown,
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
            | LoginFail | Unauthorized | JWTError(_) => StatusCode::UNAUTHORIZED,
            AlreadyTaken(..) => StatusCode::CONFLICT,
            Validation(..) | LibMultipartError(_) => StatusCode::BAD_REQUEST,
            AwsSdkError(..) | IOError(..) | HashError | ChronoParseError | Unknown | Sqlx(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl From<ModelError> for RouteError {
    fn from(_value: ModelError) -> Self {
        RouteError::Unknown
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

impl From<sqlx::Error> for RouteError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value.to_string())
    }
}

impl From<SdkError<PutObjectError>> for RouteError {
    fn from(value: SdkError<PutObjectError>) -> Self {
        Self::AwsSdkError(value.to_string())
    }
}

impl From<SdkError<DeleteObjectError>> for RouteError {
    fn from(value: SdkError<DeleteObjectError>) -> Self {
        Self::AwsSdkError(value.to_string())
    }
}

impl From<SdkError<GetObjectError>> for RouteError {
    fn from(value: SdkError<GetObjectError>) -> Self {
        Self::AwsSdkError(value.to_string())
    }
}

impl From<LibMultipartError> for RouteError {
    fn from(value: LibMultipartError) -> Self {
        RouteError::LibMultipartError(value)
    }
}

impl From<jsonwebtoken::errors::Error> for RouteError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        RouteError::JWTError(value.to_string())
    }
}

impl ToString for RouteError {
    fn to_string(&self) -> String {
        use RouteError::*;
        match self {
            AlreadyTaken(s) => format!("{} already taken", s),
            ExpiredAuthToken => format!("Auth token expired"),
            InvalidAuth => format!("Invalid auth token"),
            JWTError(j) => format!("{:?}", j),
            LoginFail => format!("Login failed"),
            MissingAuthCookie => format!("Missing auth token"),
            MissingJWTSignature => format!("Missing JWT signature"),
            Validation(s) => s.to_string(),
            LibMultipartError(m) => format!("{:?}", m),
            Unauthorized => "".to_string(),
            AwsSdkError(..) | Sqlx(..) | IOError(..) | HashError | ChronoParseError | Unknown => {
                format!("Internal error")
            }
        }
    }
}
