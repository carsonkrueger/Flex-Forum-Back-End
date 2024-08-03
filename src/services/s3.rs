use aws_sdk_s3::{
    error::SdkError,
    operation::{
        delete_object::{DeleteObjectError, DeleteObjectOutput},
        get_object::{GetObjectError, GetObjectOutput},
        put_object::{PutObjectError, PutObjectOutput},
    },
    primitives::{ByteStream, SdkBody},
    Client,
};
use axum::body::Bytes;

use crate::models::content_model::PostType;

const IMAGE_BUCKET: &str = "flexforumimages1";
const WORKOUT_BUCKET: &str = "flexforumworkouts1";

pub enum FlexForumBucket {
    ImageBucket1,
    WorkoutBucket1,
}

impl From<PostType> for String {
    fn from(value: PostType) -> Self {
        match value {
            PostType::Images => IMAGE_BUCKET,
            PostType::Workout => WORKOUT_BUCKET,
        }
        .to_string()
    }
}

pub async fn s3_upload_post(
    s3_client: &Client,
    bytes: Bytes,
    username: &str,
    post_id: i64,
    content_num: usize,
    content_type: impl Into<String>,
    post_type: PostType,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    // Result<PutObjectOutput, SdkError<PutObjectError,  Response>>
    let key = user_post_bucket_key(username, post_id, content_num);
    let sdk_body = SdkBody::from(bytes);
    let byte_stream = ByteStream::new(sdk_body);
    let res = s3_client
        .put_object()
        .bucket(post_type)
        .key(&key)
        .body(byte_stream)
        .content_type(content_type)
        .send()
        .await;
    res
}

pub async fn s3_download_post(
    s3_client: &Client,
    username: &str,
    post_id: i64,
    content_num: usize,
    bucket: PostType,
) -> Result<GetObjectOutput, SdkError<GetObjectError>> {
    let key = user_post_bucket_key(username, post_id, content_num);
    let res = s3_client.get_object().bucket(bucket).key(&key).send().await;
    res
}

pub async fn s3_delete_post(
    s3_client: &Client,
    username: &str,
    post_id: i64,
    content_num: usize,
) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>> {
    let key = user_post_bucket_key(username, post_id, content_num);
    let res = s3_client
        .delete_object()
        .bucket(IMAGE_BUCKET)
        .key(&key)
        .send()
        .await;
    res
}

pub async fn s3_upload_profile_picture(
    s3_client: &Client,
    username: &str,
    bytes: Bytes,
    content_type: impl Into<String>,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let key = format!("{}/{}", username, "profile_picture");
    let sdk_body = SdkBody::from(bytes);
    let byte_stream = ByteStream::new(sdk_body);
    let res = s3_client
        .put_object()
        .bucket(IMAGE_BUCKET)
        .key(&key)
        .body(byte_stream)
        .content_type(content_type)
        .send()
        .await;
    res
}

fn user_post_bucket_key(username: &str, post_id: i64, img_num: usize) -> String {
    format!("{}/{}/{}", username, post_id, img_num)
}
