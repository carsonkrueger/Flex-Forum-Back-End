use axum::{
    routing::{get, post, MethodRouter},
    Router,
};
use sqlx::PgPool;
use std::convert::Infallible;

use self::{hello_world::hello_world, user::create_user};

mod hello_world;
mod user;

pub trait Route<S>
where
    S: Clone + Sync + Send,
{
    fn path() -> &'static str;
    fn method_router() -> MethodRouter<S, Infallible>;
}

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/helloworld", get(hello_world))
        .route("/users", post(create_user))
        .with_state(pool)
}
