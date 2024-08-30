use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use jwt::JWT;
use lib_routes::error::{RouteError, RouterResult};

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
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> RouterResult<Self> {
        parts
            .extensions
            .get::<RouterResult<Ctx>>()
            .ok_or(RouteError::InvalidAuth)?
            .clone()
    }
}
