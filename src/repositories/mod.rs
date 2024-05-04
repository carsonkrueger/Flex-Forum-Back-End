use self::{hello_world::HelloWorldRoute, user::UserRoute};
use axum::Router;
use sqlx::PgPool;

mod hello_world;
mod user;

pub trait NestedRoute<S> {
    fn path<'a>() -> &'a str;
    fn router() -> Router<PgPool>;
}

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .nest(HelloWorldRoute::path(), HelloWorldRoute::router())
        .nest(UserRoute::path(), UserRoute::router())
        // .route("/users", post(create_user))
        // .route("/users/:id", post(get_user))
        .with_state(pool)
}
