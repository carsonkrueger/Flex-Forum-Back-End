use super::Route;
use axum::routing::{get, MethodRouter};
use std::convert::Infallible;

const PATH: &str = "/helloworld";

pub struct HelloWorld;

impl Route for HelloWorld {
    fn path() -> &'static str {
        PATH
    }
    fn method_router<S>() -> MethodRouter<S, Infallible>
    where
        S: Clone + Send + Sync + 'static,
    {
        get(hello_world)
    }
}

pub async fn hello_world() -> String {
    "hello world!".to_owned()
}
