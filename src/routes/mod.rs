use self::{hello_world::HelloWorldRoute, user_repo::UserRoute};
use axum::Router;
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
        .with_state(pool)
}
