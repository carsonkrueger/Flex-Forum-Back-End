use axum::{extract::State, routing::get, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use super::Route;

const PATH: &str = "/user";

pub struct User;

// impl<S> Route<S> for User
// where
//     S: Clone,
// {
//     fn path() -> &'static str {
//         &PATH
//     }
//     fn method_router() -> axum::routing::MethodRouter<S, std::convert::Infallible> {
//         get(mirror_user)
//     }
// }

#[derive(Deserialize, Serialize, Validate)]
pub struct MirrorUserJson {
    #[validate(length(min = 1, max = 32))]
    first_name: String,
    #[validate(length(min = 1, max = 32))]
    last_name: String,
    #[validate(email, length(min = 1, max = 255))]
    email: String,
    #[validate(length(min = 1, max = 32))]
    username: String,
    #[validate(length(min = 1, max = 64))]
    password: String,
}

pub async fn mirror_user(
    State(pool): State<PgPool>,
    Json(body): Json<MirrorUserJson>,
) -> Json<MirrorUserJson> {
    Json(body)
}
