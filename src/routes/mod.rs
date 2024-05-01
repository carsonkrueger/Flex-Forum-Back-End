use crate::{hello_world, routes::hello_world::HelloWorld};
use axum::{
    body::Body,
    routing::{get, MethodRouter},
    Router,
};
use sqlx::{Database, Pool};

mod hello_world;

pub trait Route {
    fn path() -> &'static str;
    fn method_router() -> MethodRouter;
}

pub fn create_routes<T: Database>(pool: Pool<T>) -> Router<Body> {
    Router::new().route("/", get(hello_world)).with_state(pool)
}
