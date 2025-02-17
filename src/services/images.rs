use axum::body::Bytes;
use axum_typed_multipart::FieldData;

use crate::{
    model::{
        base::BaseModelTrait,
        schemas::post_management::posts::{CreatePostModel, PostType, Posts},
    },
    route::{error::RouteResult, AppState},
};

use super::s3::S3ServiceTrait;

pub trait ImagesServiceTrait {
    async fn upload_images<BM: BaseModelTrait, S3: S3ServiceTrait>(
        username: &str,
        images: &[Option<FieldData<Bytes>>],
        description: String,
        s: &AppState,
    ) -> RouteResult<()>;
}

pub struct ImagesService;

impl ImagesServiceTrait for ImagesService {
    async fn upload_images<BM: BaseModelTrait, S3: S3ServiceTrait>(
        username: &str,
        images: &[Option<FieldData<Bytes>>],
        description: String,
        s: &AppState,
    ) -> RouteResult<()> {
        let mut tx = s.pool.begin().await?;

        let num_images = images.iter().fold(1, |x, y| {
            if let Some(_) = y {
                return x + 1;
            }
            return x;
        });
        let create_post = CreatePostModel {
            username: username.to_string(),
            num_images,
            description,
            post_type: PostType::Images,
        };
        let post = BM::insert_returning::<Posts, CreatePostModel>(create_post, &mut *tx).await?;

        for counter in 0..images.len() {
            if let Some(img) = &images[counter] {
                let res = S3::s3_upload_post(
                    &s.s3_client,
                    img.contents.clone(),
                    username,
                    post.id,
                    counter,
                    img.metadata.content_type.clone().unwrap(), // content type validated abolve
                    PostType::Images,
                )
                .await;

                if let Err(_) = res {
                    S3::s3_delete_post(&s.s3_client, username, post.id, counter - 1).await?;
                }

                res?;
            }
        }

        s.ndarray_app_state
            .lock()
            .unwrap()
            .add_post(post.id)
            .unwrap();

        tx.commit().await?;

        Ok(())
    }
}
