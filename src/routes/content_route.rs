use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Serialize;
use sqlx::types::chrono;

use crate::{
    libs::ctx::Ctx,
    models::{
        content_model::{self, get_three_older, ContentModel},
        likes_model::{get_num_likes, is_liked, LikePost, LikesModel},
    },
    services::{auth::check_username, multipart::validate_content_type, s3::s3_upload_image},
    AppState,
};

use super::{NestedRoute, RouterResult};

pub struct ContentRoute;

impl NestedRoute<AppState> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<AppState> {
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
    State(s): State<AppState>,
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
    let post_id = super::models::base::create::<ContentModel, _>(post, &s.pool).await?;
    let mut counter = 1;
    let username = ctx.jwt().username();

    let res = s3_upload_image(
        &s.s3_client,
        upload.image1.contents.clone(),
        username,
        post_id,
        counter,
    )
    .await;

    if let Some(img) = upload.image2 {
        counter += 1;
        let res = s3_upload_image(
            &s.s3_client,
            img.contents.clone(),
            username,
            post_id,
            counter,
        )
        .await;
    }

    if let Some(img) = upload.image3 {
        counter += 1;
        let res = s3_upload_image(&s.s3_client, img.contents, username, post_id, counter).await;
    }

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
    State(s): State<AppState>,
    Path(created_at): Path<chrono::DateTime<chrono::Utc>>,
) -> RouterResult<Json<Vec<PostCard>>> {
    let posts = get_three_older(&s.pool, &created_at).await?;
    let mut post_cards: Vec<PostCard> = Vec::with_capacity(3);

    for i in 0..posts.len() {
        let post_id = posts[i].id;
        let num_likes = get_num_likes(&s.pool, post_id).await?;
        let like = LikePost {
            post_id,
            username: ctx.jwt().username().to_string(),
        };
        let is_liked = is_liked(&s.pool, like).await?;
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
    State(s): State<AppState>,
    Path(post_id): Path<i64>,
) -> RouterResult<()> {
    let like: LikePost = LikePost {
        post_id,
        username: ctx.jwt().username().to_string(),
    };
    super::models::base::create::<LikesModel, LikePost>(like, &s.pool).await?;
    Ok(())
}

async fn unlike_post(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(post_id): Path<i64>,
) -> RouterResult<()> {
    super::models::base::delete_with_both::<LikesModel, _, _>(
        "post_id",
        post_id,
        "username",
        ctx.jwt().username(),
        &s.pool,
    )
    .await?;
    Ok(())
}
