use std::io::Write;

use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::libs::ctx::Ctx;

use super::{NestedRoute, RouterResult};

pub struct ContentRoute;

impl NestedRoute<PgPool> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<PgPool> {
        Router::new().route("/", post(updload_image))
    }
}

async fn updload_image(
    ctx: Ctx,
    Path(username): Path<String>,
    State(pool): State<PgPool>,
    // Json(body): Json<ImageData>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_dir = format!("./content/{}/{}", username, ctx.jwt().id());
    std::fs::create_dir_all(user_dir.clone()).unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(file_name) = field.file_name() {
            let file_path = format!("{}/{}", user_dir, file_name);
            let data = field.bytes().await.unwrap();

            // spawn_blocking to handle file write
            tokio::task::spawn_blocking(move || {
                std::fs::create_dir_all(file_path.clone()).unwrap();
                let mut file = std::fs::File::create_new(file_path).unwrap();
                file.write_all(&data).unwrap();
            })
            .await
            .unwrap();
        } else {
            println!("not file found: {:?}", field);
        }
    }

    "file created"
}
