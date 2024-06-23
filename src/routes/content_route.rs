use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, PgPool};

use crate::{
    libs::ctx::Ctx,
    models::{
        content_model::{self, get_three_older, ContentModel},
        likes_model::{get_num_likes, is_liked, LikePost},
    },
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
            .route("/images/:username", post(upload_image))
            .route("/images/:username/:post_id/:image_id", get(download))
            .route("/posts/:created_at", get(get_post_by_time))
            .route("/like/:post_id", post(like_post))
            .route("/like/:post_id", delete(unlike_post))
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

#[derive(Serialize)]
struct PostCard {
    #[serde(flatten)]
    content_model: ContentModel,
    num_likes: usize,
    is_liked: bool,
}

async fn get_post_by_time(
    ctx: Ctx,
    State(pool): State<PgPool>,
    Path(created_at): Path<chrono::DateTime<chrono::Utc>>,
) -> RouterResult<Json<Vec<PostCard>>> {
    let posts = get_three_older(&pool, &created_at).await?;
    let mut post_cards: Vec<PostCard> = Vec::with_capacity(3);

    for i in 0..posts.len() {
        let post_id = posts[i].id;
        let num_likes = get_num_likes(&pool, post_id).await?;
        let like = LikePost {
            post_id,
            username: ctx.jwt().username().to_string(),
        };
        let is_liked = is_liked(&pool, like).await?;
        let card = PostCard {
            content_model: posts[i].clone(),
            is_liked,
            num_likes,
        };
        post_cards.push(card);
    }

    Ok(Json(post_cards))
}

async fn like_post(
    ctx: Ctx,
    State(pool): State<PgPool>,
    Path(post_id): Path<i64>,
) -> RouterResult<()> {
    let like: LikePost = LikePost {
        post_id,
        username: ctx.jwt().username().to_string(),
    };
    super::models::likes_model::create(&pool, like).await?;
    Ok(())
}

async fn unlike_post(
    ctx: Ctx,
    State(pool): State<PgPool>,
    Path(post_id): Path<i64>,
) -> RouterResult<()> {
    let like: LikePost = LikePost {
        post_id,
        username: ctx.jwt().username().to_string(),
    };
    super::models::likes_model::delete(&pool, like).await?;
    Ok(())
}
