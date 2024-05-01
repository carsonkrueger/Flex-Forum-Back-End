use super::Route;
use axum::routing::{get, MethodRouter};

const PATH: &str = "/helloworld";

pub struct HelloWorld;

impl Route for HelloWorld {
    fn path() -> &'static str {
        PATH
    }
    fn method_router() -> MethodRouter {
        get(hello_world)
    }
}

pub async fn hello_world() -> String {
    "hello world!".to_owned()
}
