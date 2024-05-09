use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};

use crate::libs::{ctx::Ctx, jwt::JWT};
use crate::routes::{Error, Result};

pub const AUTH_TOKEN: &'static str = "auth_token";

/// Enforces auth Ctx within extensions and validates the jwt
pub async fn validate_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    ctx?.jwt().validate_token()?;
    Ok(next.run(req).await)
}

/// Creates Ctx from cookies and inserts into Extensions then calls next layer.
/// Returns Err if missing or invalid JWT.
pub async fn ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    let token_str = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match token_str
        .ok_or(Error::MissingAuthCookie)
        .and_then(JWT::parse_token)
    {
        Ok(jwt) => Ok(Ctx::new(jwt)),
        Err(e) => Err(e),
    };

    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::MissingAuthCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    req.extensions_mut().insert(result_ctx);
    Ok(next.run(req).await)
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
