use self::{hello_world::HelloWorldRoute, user_repo::UserRoute};
use crate::middleware::auth::{ctx_resolver, validate_auth};
use axum::{middleware::from_fn, Router};
use sqlx::PgPool;

mod hello_world;
mod user_repo;

pub trait NestedRoute<S> {
    fn path<'a>() -> &'a str;
    fn router() -> Router<S>;
}

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .nest(HelloWorldRoute::path(), HelloWorldRoute::router())
        .nest(UserRoute::path(), UserRoute::router())
        .layer(from_fn(validate_auth))
        .layer(from_fn(ctx_resolver))
        .with_state(pool)
}
