use crate::AppState;

use super::NestedRoute;
use axum::{routing::get, Router};

pub struct HelloWorldRoute;

impl<'a> NestedRoute<AppState> for HelloWorldRoute {
    const PATH: &'static str = "/helloworld";
    fn router() -> axum::Router<AppState> {
        Router::new().route("/", get(hello_world))
    }
}

pub async fn hello_world() -> String {
    "hello world!!1!".to_owned()
}
