use axum::{routing::get, Router};
use sqlx::PgPool;

use super::NestedRoute;

pub struct HelloWorldRoute;

impl NestedRoute<PgPool> for HelloWorldRoute {
    fn path<'a>() -> &'a str {
        "/helloworld"
    }
    fn router() -> axum::Router<PgPool> {
        Router::new().route("/", get(hello_world))
    }
}

pub async fn hello_world() -> String {
    "hello world!".to_owned()
}
