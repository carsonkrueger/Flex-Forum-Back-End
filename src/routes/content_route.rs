use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Deserialize;
use sqlx::{types::chrono, PgPool};

use crate::{
    libs::ctx::Ctx,
    models::content_model::{self, get_five, ContentModel},
    services::{
        auth::check_username,
        multipart::{create_file, validate_content_type},
    },
};

use super::{NestedRoute, RouterResult};

pub struct ContentRoute;

impl NestedRoute<PgPool> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<PgPool> {
        Router::new()
            .route("/:username", post(upload_image))
            .route("/:username/:post_id/:image_id", get(download))
            .route("/", get(get_post_by_time))
    }
}

#[derive(TryFromMultipart)]
struct UploadImageMulipart {
    image1: FieldData<Bytes>,
    image2: Option<FieldData<Bytes>>,
    image3: Option<FieldData<Bytes>>,
    description: Option<String>,
}

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg"];
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

    let mut counter = 1;
    if let Some(_) = upload.image2 {
        counter += 1;
    }
    if let Some(_) = upload.image3 {
        counter += 1;
    }

    // let transaction = pool.begin().await?;
    let post = content_model::CreatePostModel {
        username: ctx.jwt().username().to_string(),
        num_images: counter,
        description: upload.description,
    };
    let post_id = content_model::create(&pool, post).await?;

    let user_dir = format!("{}/{}/{}", CONTENT_IMAGE_PATH, username, post_id);
    std::fs::create_dir_all(user_dir.clone()).unwrap();

    tokio::task::spawn_blocking(move || -> RouterResult<()> {
        let mut counter = 1;
        let file_path1 = format!("{}/{}.jpeg", user_dir, counter);
        counter += 1;
        create_file(&upload.image1, file_path1)?;

        if let Some(fd) = upload.image2 {
            let file_path2 = format!("{}/{}.jpeg", user_dir, counter);
            counter += 1;
            create_file(&fd, file_path2)?;
        }

        if let Some(fd) = upload.image3 {
            let file_path3 = format!("{}/{}.jpeg", user_dir, counter);
            counter += 1;
            create_file(&fd, file_path3)?;
        }

        Ok(())
    })
    .await
    .unwrap()?;

    Ok("file created".to_string())
}

async fn download(
    _ctx: Ctx,
    Path((username, post_id, image_id)): Path<(String, i64, i64)>,
) -> RouterResult<Body> {
    let image_path = format!(
        "{}/{}/{}/{}.jpeg",
        CONTENT_IMAGE_PATH, username, post_id, image_id
    );

    let file = tokio::fs::File::open(image_path).await?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let data = Body::from_stream(stream);

    Ok(data)
}

#[derive(Deserialize)]
struct PostByTime {
    pub created_at: chrono::DateTime<chrono::Utc>,
}

async fn get_post_by_time(
    State(pool): State<PgPool>,
    Json(body): Json<PostByTime>,
) -> RouterResult<Json<Vec<ContentModel>>> {
    println!("{:?}", body.created_at);
    let posts = get_five(&pool, &body.created_at).await?;
    println!("{:?}", posts);
    Ok(Json(posts))
    // todo!()
}
