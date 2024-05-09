use axum::{routing::get, Router};
use sqlx::PgPool;

use super::NestedRoute;

pub struct HelloWorldRoute;

impl NestedRoute<PgPool> for HelloWorldRoute {
    const PATH: &'static str = "/helloworld";
    fn router() -> axum::Router<PgPool> {
        Router::new().route("/", get(hello_world))
    }
}

pub async fn hello_world() -> String {
    "hello world!!1!".to_owned()
}
