use axum::{body::Body, response::Response};

pub async fn logger(res: Response<Body>) -> Response {
    println!("{:?}", res.status());
    println!("{:?}", res);
    res
}
