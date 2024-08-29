use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use validator::Validate;

use crate::{
    libs::ctx::Ctx,
    models::{
        base::{self},
        content_model::{self, get_ten_unseen_older, get_three_older, ContentModel, PostType},
        following_model::{is_following, FollowingModel},
        likes_model::{get_num_likes, is_liked, LikePost, LikesModel},
        profile_picture_model::ProfilePictureModel,
        seen_posts_model::seen,
        user_model::get_user_id,
    },
    services::{
        multipart::validate_content_type,
        posts::sort_by_predicted,
        s3::{s3_delete_post, s3_download_post, s3_upload_post, s3_upload_profile_picture},
    },
    AppState,
};

use super::{NestedRoute, RouteError, RouterResult};

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
    description: Option<String>,
}

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/jpg"];
const JSON_CONTENT_TYPE: &'static str = "application/json";

async fn upload_images_post(
    ctx: Ctx,
    State(s): State<AppState>,
    TypedMultipart(upload): TypedMultipart<UploadImageMulipart>,
) -> RouterResult<StatusCode> {
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

    let mut transaction = s.pool.begin().await?;

    // let transaction = pool.begin().await?;
    let post = content_model::CreatePostModel {
        username: ctx.jwt().username().to_string(),
        num_images: counter,
        description: upload.description,
        post_type: PostType::Images,
    };
    let post_id =
        super::models::base::create_with_transaction::<ContentModel, _>(post, &mut transaction)
            .await?;
    let mut counter = 1;
    let username = ctx.jwt().username();

    s3_upload_post(
        &s.s3_client,
        upload.image1.contents.clone(),
        username,
        post_id,
        counter,
        upload.image1.metadata.content_type.unwrap(), // content type validated abolve
        PostType::Images,
    )
    .await?;

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

        if let Err(_) = res {
            s3_delete_post(&s.s3_client, username, post_id, counter - 1).await?;
        }

        res?;
    }

    if let Some(img) = upload.image3 {
        counter += 1;
        let res = s3_upload_post(
            &s.s3_client,
            img.contents,
            username,
            post_id,
            counter,
            img.metadata.content_type.unwrap(), // content type validated above
            PostType::Images,
        )
        .await;

        if let Err(_) = res {
            s3_delete_post(&s.s3_client, username, post_id, counter - 2).await?;
            s3_delete_post(&s.s3_client, username, post_id, counter - 1).await?;
        }

        res?;
    }

    s.ndarray_app_state
        .lock()
        .unwrap()
        .add_post(post_id)
        .unwrap();

    transaction.commit().await?;

    Ok(StatusCode::CREATED)
}

async fn download(
    _ctx: Ctx,
    Path((post_type, username, post_id, content_id)): Path<(PostType, String, i64, i64)>,
    State(s): State<AppState>,
) -> RouterResult<Body> {
    let res = s3_download_post(
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
    description: Option<String>,
}

async fn upload_workout_post(
    ctx: Ctx,
    State(s): State<AppState>,
    body: Json<UploadWorkout>,
) -> RouterResult<StatusCode> {
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

    let mut transaction = s.pool.begin().await?;

    let post_id =
        super::models::base::create_with_transaction::<ContentModel, _>(post, &mut transaction)
            .await?;

    //let byte_slice = unsafe { any_as_u8_slice(&body.workout) };
    // let bytes = axum::body::Bytes::copy_from_slice(byte_slice);
    let json_string = serde_json::to_string(&body.workout).unwrap();
    let bytes = Bytes::from(json_string);

    s3_upload_post(
        &s.s3_client,
        bytes,
        ctx.jwt().username(),
        post_id,
        1,
        JSON_CONTENT_TYPE,
        PostType::Workout,
    )
    .await?;

    transaction.commit().await?;

    Ok(StatusCode::CREATED)
}

#[derive(Serialize, Debug)]
struct PostCard {
    #[serde(flatten)]
    content_model: ContentModel,
    num_likes: usize,
    is_liked: bool,
    is_following: bool,
}

async fn get_post_by_time(
    ctx: Ctx,
    State(s): State<AppState>,
    Path(created_at): Path<NaiveDateTime>,
) -> RouterResult<Json<Vec<PostCard>>> {
    // let posts = get_three_older(&s.pool, &created_at).await?;
    let mut posts = get_ten_unseen_older(&s.pool, &created_at, ctx.jwt().username()).await?;
    let user_id = get_user_id(ctx.jwt().username(), &s.pool).await?.unwrap();
    // .unwrap_or(Err(RouteError::Unauthorized)?);

    if posts.len() > 0 {
        sort_by_predicted(&mut posts, &s, 3, user_id);

        // mark all posts as seen so that they do not get recommended again
        for p in &posts {
            seen(&s.pool, ctx.jwt().username(), p.id).await?;
        }
    }
    // if the posts length is 0 then they have seen all recommended posts, so just give them older seen content again
    else {
        posts = get_three_older(&s.pool, &created_at).await?;
    }

    let mut post_cards: Vec<PostCard> = Vec::with_capacity(posts.len());

    for i in 0..posts.len() {
        let post_id = posts[i].id;
        let num_likes = get_num_likes(&s.pool, post_id).await?;
        let like = LikePost {
            post_id,
            username: ctx.jwt().username().to_string(),
        };
        let is_liked = is_liked(&s.pool, like).await?;
        // let is_following = false;
        let is_following = is_following(&s.pool, ctx.jwt().username(), &posts[i].username).await?;
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
) -> RouterResult<()> {
    let like: LikePost = LikePost {
        post_id,
        username: ctx.jwt().username().to_string(),
    };
    super::models::base::create::<LikesModel, LikePost>(like, &s.pool).await?;
    seen(&s.pool, ctx.jwt().username(), post_id).await?;
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

#[derive(TryFromMultipart)]
struct UploadProfileImageMulipart {
    image: FieldData<Bytes>,
}

async fn upload_profile_picture(
    ctx: Ctx,
    State(s): State<AppState>,
    TypedMultipart(upload): TypedMultipart<UploadProfileImageMulipart>,
) -> RouterResult<()> {
    validate_content_type(&upload.image, IMAGE_CONTENT_TYPES)?;

    let model = ProfilePictureModel {
        id: 0,
        username: ctx.jwt().username().to_string(),
    };

    let mut transaction = s.pool.begin().await?;

    let items = base::get_all_with_transaction::<ProfilePictureModel, ProfilePictureModel>(
        &mut transaction,
        None,
    )
    .await?;

    if items.len() == 0 {
        base::create_with_transaction::<ProfilePictureModel, _>(model, &mut transaction).await?;
    }

    s3_upload_profile_picture(
        &s.s3_client,
        ctx.jwt().username(),
        upload.image.contents,
        upload.image.metadata.content_type.unwrap(), // content type validated above
    )
    .await?;

    transaction.commit().await?;

    Ok(())
}
