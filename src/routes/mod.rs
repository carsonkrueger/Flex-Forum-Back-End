use axum::{routing::MethodRouter, Router};
use sqlx::{Database, Pool};
use std::convert::Infallible;

use self::hello_world::HelloWorld;

mod hello_world;

pub trait Route {
    fn path() -> &'static str;
    fn method_router<S>() -> MethodRouter<S, Infallible>
    where
        S: Clone + Send + Sync + 'static;
}

pub fn create_routes<T: Database>(pool: Pool<T>) -> Router {
    Router::new()
        .route(HelloWorld::path(), HelloWorld::method_router())
        .with_state(pool)
}
