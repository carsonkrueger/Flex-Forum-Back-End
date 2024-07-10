use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use validator::Validate;

use crate::{
    libs::ctx::Ctx,
    models::{
        content_model::{self, get_three_older, ContentModel, PostType},
        likes_model::{get_num_likes, is_liked, LikePost, LikesModel},
    },
    services::{
        auth::check_username,
        multipart::validate_content_type,
        s3::{s3_download_image, s3_upload_post},
    },
    AppState,
};

use super::{bytes::any_as_u8_slice, NestedRoute, RouteError, RouterResult};

pub struct ContentRoute;

impl NestedRoute<AppState> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/images", post(upload_images_post))
            .route("/images/:username/:post_id/:image_id", get(download))
            .route("/workouts", post(upload_workout_post))
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

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/jpg"];

async fn upload_images_post(
    ctx: Ctx,
    State(s): State<AppState>,
    TypedMultipart(upload): TypedMultipart<UploadImageMulipart>,
) -> RouterResult<String> {
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
        post_type: PostType::Images,
    };
    let post_id = super::models::base::create::<ContentModel, _>(post, &s.pool).await?;
    let mut counter = 1;
    let username = ctx.jwt().username();

    let res = s3_upload_post(
        &s.s3_client,
        upload.image1.contents.clone(),
        username,
        post_id,
        counter,
        upload.image1.metadata.content_type.unwrap(), // content type validated abolve
        PostType::Images,
    )
    .await;

    if let Some(img) = upload.image2 {
        counter += 1;
        let res = s3_upload_post(
            &s.s3_client,
            img.contents.clone(),
            username,
            post_id,
            counter,
            img.metadata.content_type.unwrap(), // content type validated abolve
            PostType::Images,
        )
        .await;
    }

    if let Some(img) = upload.image3 {
        counter += 1;
        let res = s3_upload_post(
            &s.s3_client,
            img.contents,
            username,
            post_id,
            counter,
            img.metadata.content_type.unwrap(), // content type validated abolve
            PostType::Images,
        )
        .await;
    }

    Ok("file created".to_string())
}

async fn download(
    _ctx: Ctx,
    Path((username, post_id, image_id)): Path<(String, i64, i64)>,
    State(s): State<AppState>,
) -> RouterResult<Body> {
    let res = s3_download_image(&s.s3_client, &username, post_id, image_id as usize).await;

    let stream = tokio_util::io::ReaderStream::new(res.unwrap().body.into_async_read());
    let data = Body::from_stream(stream);

    Ok(data)
}

#[derive(Deserialize, Serialize)]
pub struct Exercise {
    exercise_name: String,
    num_sets: i32,
    num_reps: i32,
    timer: u32,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct Workout {
    #[validate(length(min = 1, max = 64))]
    workout_name: String,
    #[validate(length(min = 1, max = 20))]
    exercises: Vec<Exercise>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UploadWorkout {
    workout: Workout,
    #[validate(length(max = 1000))]
    description: Option<String>,
}

async fn upload_workout_post(
    ctx: Ctx,
    State(s): State<AppState>,
    body: Json<UploadWorkout>,
) -> RouterResult<()> {
    if let Err(e) = body.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }

    if let Err(e) = body.workout.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }

    let post = content_model::CreatePostModel {
        username: ctx.jwt().username().to_string(),
        num_images: 0,
        description: body.description.clone(),
        post_type: PostType::Workout,
    };

    let post_id = super::models::base::create::<ContentModel, _>(post, &s.pool).await?;

    let byte_slice = unsafe { any_as_u8_slice(&body.workout) };
    let bytes = axum::body::Bytes::copy_from_slice(byte_slice);

    s3_upload_post(
        &s.s3_client,
        bytes,
        ctx.jwt().username(),
        post_id,
        1,
        "application/json",
        PostType::Workout,
    )
    .await;

    Ok(())
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
