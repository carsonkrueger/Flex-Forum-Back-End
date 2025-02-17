use aws_sdk_s3::Client;
use axum::body::Bytes;
use sqlx::PgPool;

use crate::{
    model::{
        base::BaseModelTrait,
        schemas::post_management::posts::{CreatePostModel, PostType, Posts},
    },
    route::{
        error::RouteResult,
        private_routes::content::{UploadWorkout, JSON_CONTENT_TYPE},
    },
};

use super::s3::{S3Service, S3ServiceTrait};

pub trait WorkoutsServiceTrait {
    async fn upload_workout<BM: BaseModelTrait>(
        username: &str,
        body: UploadWorkout,
        post: CreatePostModel,
        pool: PgPool,
        s3: Client,
    ) -> RouteResult<()>;
}

pub struct WorkoutsService;

impl WorkoutsServiceTrait for WorkoutsService {
    async fn upload_workout<BM: BaseModelTrait>(
        username: &str,
        body: UploadWorkout,
        post: CreatePostModel,
        pool: PgPool,
        s3: Client, // s: &AppState,
    ) -> RouteResult<()> {
        let mut tx = pool.begin().await?;

        let post = BM::insert_returning::<Posts, CreatePostModel>(post, &mut *tx).await?;

        let json_string = serde_json::to_string(&body.workout).unwrap();
        let bytes = Bytes::from(json_string);

        S3Service::s3_upload_post(
            &s3,
            bytes,
            username,
            post.id,
            1,
            JSON_CONTENT_TYPE,
            PostType::Workout,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
