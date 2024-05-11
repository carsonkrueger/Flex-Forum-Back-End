use std::env;

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use hash_lib::schemes::argon2_v01::Argon2V01;
use once_cell::sync::Lazy;
use tower_cookies::{Cookie, Cookies};

use crate::libs::{ctx::Ctx, jwt::JWT};
use crate::routes::{RouteError, RouterResult};

pub const AUTH_TOKEN: &'static str = "auth_token";
pub const JWT_SECRET: Lazy<String> = Lazy::new(get_jwt_secret);

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect("Could not get JWT_SECRET")
}

/// Enforces auth Ctx within extensions and validates the jwt
pub async fn validate_auth(
    ctx: RouterResult<Ctx>,
    req: Request<Body>,
    next: Next,
) -> RouterResult<Response> {
    ctx?.jwt().validate_token(&JWT_SECRET, &Argon2V01)?;
    Ok(next.run(req).await)
}

/// Creates Ctx from cookies and inserts into Extensions then calls next layer.
/// Returns Err if missing or invalid JWT.
pub async fn ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> RouterResult<Response> {
    let token_str = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match token_str
        .ok_or(RouteError::MissingAuthCookie)
        .and_then(JWT::parse_token)
    {
        Ok(jwt) => Ok(Ctx::new(jwt)),
        Err(e) => Err(e),
    };

    if result_ctx.is_err() && !matches!(result_ctx, Err(RouteError::MissingAuthCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    req.extensions_mut().insert(result_ctx);
    Ok(next.run(req).await)
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
