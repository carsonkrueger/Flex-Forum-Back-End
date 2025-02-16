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
        base::{self},
        schemas::{
            post_management::{
                following::is_following,
                likes::{get_num_likes, is_liked, LikePost, Likes, LikesIden},
                posts::{
                    get_ten_unseen_older, get_three_older, sort_by_predicted, CreatePostModel,
                    PostType, Posts,
                },
                seen_posts::seen,
            },
            user_management::{profile_pictures::ProfilePicture, users::get_user_by_username},
        },
    },
    route::{
        error::{RouteError, RouteResult},
        NestedRoute,
    },
    services::{
        images::ImagesService,
        s3::{S3Service, S3ServiceTrait},
    },
    util::ctx::Ctx,
    AppState,
};

pub struct ContentRoute;

impl NestedRoute<AppState> for ContentRoute {
    const PATH: &'static str = "/content";
    fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/images", post(upload_images_post))
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
const JSON_CONTENT_TYPE: &'static str = "application/json";

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

    ImagesService::upload_images(ctx.jwt().username(), &images, upload.description, &s).await?;

    Ok(StatusCode::CREATED)
}

async fn download(
    _ctx: Ctx,
    Path((post_type, username, post_id, content_id)): Path<(PostType, String, i64, i64)>,
    State(s): State<AppState>,
) -> RouteResult<Body> {
    let res = S3Service::s3_download_post(
        &s.s3_client,
        &username,
        post_id,
        content_id as usize,
        post_type,
    )
    .await?;

    let stream = tokio_util::io::ReaderStream::new(res.body.into_async_read());
    let data = Body::from_stream(stream);

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
    workout: Workout,
    #[validate(length(max = 1000))]
    description: String,
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

    let mut tx = s.pool.begin().await?;

    let post = base::insert_returning::<Posts, CreatePostModel>(post, &mut *tx).await?;

    //let byte_slice = unsafe { any_as_u8_slice(&body.workout) };
    // let bytes = axum::body::Bytes::copy_from_slice(byte_slice);
    let json_string = serde_json::to_string(&body.workout).unwrap();
    let bytes = Bytes::from(json_string);

    S3Service::s3_upload_post(
        &s.s3_client,
        bytes,
        ctx.jwt().username(),
        post.id,
        1,
        JSON_CONTENT_TYPE,
        PostType::Workout,
    )
    .await?;

    tx.commit().await?;

    Ok(StatusCode::CREATED)
}

#[derive(Serialize, Debug)]
struct PostCard {
    #[serde(flatten)]
    content_model: Posts,
    num_likes: usize,
    is_liked: bool,
    is_following: bool,
}

async fn get_post_by_time(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(created_at): Path<NaiveDateTime>,
) -> RouteResult<Json<Vec<PostCard>>> {
    let pool = &mut *s.pool.acquire().await?;

    let mut posts = get_ten_unseen_older(&s.pool, &created_at, ctx.jwt().username()).await?;
    let user = get_user_by_username(ctx.jwt().username(), pool)
        .await?
        .unwrap();
    // .unwrap_or(Err(RouteError::Unauthorized)?);

    if posts.len() > 0 {
        sort_by_predicted(&mut posts, &s, 3, user.id);

        // Mark all posts as seen so that they do not get recommended again.
        // Will likely change in the future so that interactions will only count as seen, or number of times recommended.
        for p in &posts {
            seen(&s.pool, ctx.jwt().username(), p.id).await?;
        }
    }
    // if the posts length is 0 then they have seen all recommended posts, so just give them older already seen content again
    else {
        posts = get_three_older(&s.pool, &created_at).await?;
    }

    let mut post_cards: Vec<PostCard> = Vec::with_capacity(posts.len());

    for i in 0..posts.len() {
        let post_id = posts[i].id;
        let num_likes = get_num_likes(pool, post_id).await? as usize;
        let like = LikePost {
            post_id,
            username: ctx.jwt().username().to_string(),
        };
        let is_liked = is_liked(pool, like.post_id, &user.username).await?;
        let is_following = is_following(pool, ctx.jwt().username(), &posts[i].username).await?;
        let card = PostCard {
            content_model: posts[i].clone(),
            is_liked,
            num_likes,
            is_following,
        };
        post_cards.push(card);
    }

    Ok(Json(post_cards))
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
    base::insert_returning::<Likes, LikePost>(like, &mut *s.pool.acquire().await?).await?;
    seen(&s.pool, ctx.jwt().username(), post_id).await?;
    Ok(())
}

async fn unlike_post(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(post_id): Path<i64>,
) -> RouteResult<()> {
    base::delete_one_with_both::<Likes>(
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

    let items = base::get_all::<ProfilePicture, ProfilePicture>(&mut *transaction).await?;

    if items.len() == 0 {
        base::insert_returning::<ProfilePicture, ProfilePicture>(
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
