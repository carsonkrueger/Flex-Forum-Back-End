use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;

use crate::lib::{ctx::Ctx, error::Error, error::Result};

const AUTH_TOKEN: &'static str = "auth_token";

/// Enforces auth Ctx
pub async fn require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    ctx?;
    Ok(next.run(req).await)
}

/// creates Ctx from cookies
pub async fn ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    // let token = cookies.get(AUTH_TOKEN)
    todo!()
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::InvalidAuth)?
            .clone()
    }
}
