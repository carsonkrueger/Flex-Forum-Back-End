use std::io::Write;

use axum::{
    body::Bytes,
    extract::{Path, State},
    routing::post,
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use sqlx::PgPool;

use crate::{libs::ctx::Ctx, services::auth::check_username};

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

    let user_dir = format!("./content/{}", username);
    std::fs::create_dir_all(user_dir.clone()).unwrap();

    tokio::task::spawn_blocking(move || {
        let file_path1 = format!(
            "{}/{}",
            user_dir,
            upload.image1.metadata.file_name.clone().unwrap()
        );
        create_file(&upload.image1, file_path1);

        if let Some(fd) = upload.image2 {
            let file_path2 = format!("{}/{}", user_dir, fd.metadata.file_name.clone().unwrap());
            create_file(&fd, file_path2);
        }

        if let Some(fd) = upload.image3 {
            let file_path3 = format!("{}/{}", user_dir, fd.metadata.file_name.clone().unwrap());
            create_file(&fd, file_path3);
        }
    })
    .await
    .unwrap();

    Ok("file created".to_string())
}

fn create_file(field_data: &FieldData<Bytes>, file_path: String) {
    let mut file = std::fs::File::options()
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap();
    file.write_all(&field_data.contents).unwrap();
}

fn validate_content_type(
    field_data: &FieldData<Bytes>,
    content_types: &[&str],
) -> RouterResult<()> {
    if let Some(s) = &field_data.metadata.content_type {
        for &c in content_types {
            if c == s.as_str() {
                return Ok(());
            }
        }
    }
    Err(RouteError::InvalidContentType(
        field_data.metadata.name.clone().unwrap_or("".to_string()),
    ))
}
