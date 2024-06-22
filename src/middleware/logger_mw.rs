use axum::{body::Body, response::Response};

/// Logs info about responses - Errors, etc.
pub async fn logger(res: Response<Body>) -> Response {
    println!("{:?}", res.status());
    println!("{:?}\n", res);
    res
}
