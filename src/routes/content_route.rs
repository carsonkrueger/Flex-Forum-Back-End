use std::io::Write;

use axum::{
    body::Bytes,
    extract::{Path, State},
    routing::post,
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use sqlx::PgPool;

use crate::{
    libs::ctx::Ctx,
    services::{
        auth::check_username,
        multipart::{create_file, validate_content_type},
    },
};

use super::{NestedRoute, RouteError, RouterResult};

pub struct ContentRoute;

impl NestedRoute<PgPool> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<PgPool> {
        Router::new().route("/:username", post(upload_image))
    }
}

#[derive(TryFromMultipart)]
struct UploadImageMulipart {
    image1: FieldData<Bytes>,
    image2: Option<FieldData<Bytes>>,
    image3: Option<FieldData<Bytes>>,
    description: Option<String>,
}

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/jpg"];
const CONTENT_IMAGE_PATH: &str = "./content/images";

async fn upload_image(
    ctx: Ctx,
    Path(username): Path<String>,
    State(pool): State<PgPool>,
    TypedMultipart(upload): TypedMultipart<UploadImageMulipart>,
) -> RouterResult<String> {
    check_username(&username, ctx.jwt())?;

    validate_content_type(&upload.image1, IMAGE_CONTENT_TYPES)?;
    if let Some(fd) = &upload.image2 {
        validate_content_type(fd, IMAGE_CONTENT_TYPES)?;
    }
    if let Some(fd) = &upload.image3 {
        validate_content_type(fd, IMAGE_CONTENT_TYPES)?;
    }

    let user_dir = format!("{}/{}", CONTENT_IMAGE_PATH, username);
    std::fs::create_dir_all(user_dir.clone()).unwrap();

    tokio::task::spawn_blocking(move || -> RouterResult<()> {
        let file_path1 = format!(
            "{}/{}",
            user_dir,
            upload.image1.metadata.file_name.as_ref().unwrap()
        );
        create_file(&upload.image1, file_path1)?;

        if let Some(fd) = upload.image2 {
            let file_path2 = format!("{}/{}", user_dir, fd.metadata.file_name.as_ref().unwrap());
            create_file(&fd, file_path2)?;
        }

        if let Some(fd) = upload.image3 {
            let file_path3 = format!("{}/{}", user_dir, fd.metadata.file_name.as_ref().unwrap());
            create_file(&fd, file_path3)?;
        }

        Ok(())
    })
    .await
    .unwrap()?;

    Ok("file created".to_string())
}

struct DownloadPost {
    username: String,
    post_id: i64,
}

async fn download(
    ctx: Ctx,
    Path(DownloadPost { username, post_id }): Path<DownloadPost>,
) -> RouterResult<()> {
    let post_dir = format!("{}/{}/{}", CONTENT_IMAGE_PATH, username, post_id);
    // let files_names = std::fs::read_dir(post_dir).unwrap().collect::<();
    todo!()
}
