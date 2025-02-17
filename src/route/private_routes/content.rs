use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use chrono::NaiveDateTime;
use lib_multipart::validate_content_type;
// use ctx::Ctx;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    model::{
        base::{BaseModel, BaseModelTrait},
        schemas::{
            post_management::{
                likes::{LikePost, Likes, LikesIden},
                posts::{CreatePostModel, PostType},
                seen_posts::seen,
            },
            user_management::profile_pictures::ProfilePicture,
        },
    },
    route::{
        error::{RouteError, RouteResult},
        NestedRoute,
    },
    services::{
        images::{ImagesService, ImagesServiceTrait},
        posts::{GetPostSummary, PostsService, PostsServiceTrait},
        s3::{S3Service, S3ServiceTrait},
        workouts::{WorkoutsService, WorkoutsServiceTrait},
    },
    util::ctx::Ctx,
    AppState,
};

pub struct ContentRoute;

impl NestedRoute<AppState> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<AppState> {
        Router::new()
            // .route("/images", post(upload_images_post))
            .route("/:post_type/:username/:post_id/:content_id", get(download))
            .route("/workouts", post(upload_workout_post))
            .route("/posts/:created_at", get(get_post_by_time))
            .route("/like/:post_id", post(like_post))
            .route("/like/:post_id", delete(unlike_post))
            .route("/profile-picture", post(upload_profile_picture))
    }
}

#[derive(TryFromMultipart)]
struct UploadImageMulipart {
    image1: FieldData<Bytes>,
    image2: Option<FieldData<Bytes>>,
    image3: Option<FieldData<Bytes>>,
    description: String,
}

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/jpg"];
pub const JSON_CONTENT_TYPE: &'static str = "application/json";

async fn upload_images_post(
    ctx: Ctx,
    State(s): State<AppState>,
    TypedMultipart(upload): TypedMultipart<UploadImageMulipart>,
) -> RouteResult<StatusCode> {
    validate_content_type(&upload.image1, IMAGE_CONTENT_TYPES)?;
    if let Some(fd) = &upload.image2 {
        validate_content_type(fd, IMAGE_CONTENT_TYPES)?;
    }
    if let Some(fd) = &upload.image3 {
        validate_content_type(fd, IMAGE_CONTENT_TYPES)?;
    }

    let images: &[Option<FieldData<Bytes>>] = &[Some(upload.image1), upload.image2, upload.image3];

    ImagesService::upload_images::<BaseModel, S3Service>(
        ctx.jwt().username(),
        &images,
        upload.description,
        &s,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

async fn download(
    _ctx: Ctx,
    Path((post_type, username, post_id, content_id)): Path<(PostType, String, i64, i64)>,
    State(s): State<AppState>,
) -> RouteResult<Body> {
    let data = S3Service::s3_download_post(
        &s.s3_client,
        &username,
        post_id,
        content_id as usize,
        post_type,
    )
    .await?;

    Ok(data)
}

#[derive(Deserialize, Serialize)]
pub struct Exercise {
    preset_id: i64,
    num_sets: i32,
    num_reps: i32,
    timer: Option<u32>,
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
    pub workout: Workout,
    #[validate(length(max = 1000))]
    pub description: String,
}

async fn upload_workout_post(
    ctx: Ctx,
    State(s): State<AppState>,
    body: Json<UploadWorkout>,
) -> RouteResult<StatusCode> {
    if let Err(e) = body.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }
    if let Err(e) = body.workout.validate() {
        return Err(RouteError::Validation(e.to_string()));
    }

    let post = CreatePostModel {
        username: ctx.jwt().username().to_string(),
        num_images: 0,
        description: body.description.clone(),
        post_type: PostType::Workout,
    };

    WorkoutsService::upload_workout::<BaseModel>(
        ctx.jwt().username(),
        body.0,
        post,
        s.pool.clone(),
        s.s3_client,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

async fn get_post_by_time(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(created_at): Path<NaiveDateTime>,
) -> RouteResult<Json<Vec<GetPostSummary>>> {
    let conn = &mut *s.pool.acquire().await?;

    let post_summaries = PostsService::get_ten_posts::<BaseModel, PostsService>(
        ctx.jwt().username(),
        &created_at,
        &s,
        conn,
    )
    .await?;

    Ok(Json(post_summaries))
}

async fn like_post(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(post_id): Path<i64>,
) -> RouteResult<()> {
    let like = LikePost {
        post_id,
        username: ctx.jwt().username().to_string(),
    };
    BaseModel::insert_returning::<Likes, LikePost>(like, &mut *s.pool.acquire().await?).await?;
    seen(&s.pool, ctx.jwt().username(), post_id).await?;
    Ok(())
}

async fn unlike_post(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(post_id): Path<i64>,
) -> RouteResult<()> {
    BaseModel::delete_one_with_both::<Likes>(
        LikesIden::PostId,
        post_id.into(),
        LikesIden::Username,
        ctx.jwt().username().into(),
        &mut *s.pool.acquire().await?,
    )
    .await?;
    Ok(())
}

#[derive(TryFromMultipart)]
struct UploadProfileImageMulipart {
    image: FieldData<Bytes>,
}

async fn upload_profile_picture(
    ctx: Ctx,
    State(s): State<AppState>,
    TypedMultipart(upload): TypedMultipart<UploadProfileImageMulipart>,
) -> RouteResult<()> {
    validate_content_type(&upload.image, IMAGE_CONTENT_TYPES)?;

    let profile_picture = ProfilePicture {
        id: 0,
        username: ctx.jwt().username().to_string(),
    };

    let mut transaction = s.pool.begin().await?;

    let items = BaseModel::get_all::<ProfilePicture, ProfilePicture>(&mut *transaction).await?;

    if items.len() == 0 {
        BaseModel::insert_returning::<ProfilePicture, ProfilePicture>(
            profile_picture,
            &mut *transaction,
        )
        .await?;
    }

    S3Service::s3_upload_profile_picture(
        &s.s3_client,
        ctx.jwt().username(),
        upload.image.contents,
        upload.image.metadata.content_type.unwrap(), // content type validated above
    )
    .await?;

    transaction.commit().await?;

    Ok(())
}
