use super::jwt::JWT;
use crate::route::error::{RouteError, RouteResult};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

#[derive(Clone, Debug)]
pub struct Ctx {
    jwt: JWT,
}

impl Ctx {
    pub fn new(jwt: JWT) -> Self {
        Self { jwt }
    }
    pub fn jwt(&self) -> &JWT {
        &self.jwt
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = RouteError;

    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> RouteResult<Self> {
        parts
            .extensions
            .get::<RouteResult<Ctx>>()
            .ok_or(RouteError::InvalidAuth)?
            .clone()
    }
}
