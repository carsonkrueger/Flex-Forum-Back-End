use aws_sdk_s3::{
    error::SdkError,
    operation::put_object::{PutObjectError, PutObjectOutput, builders::},
    primitives::{ByteStream, SdkBody},
    Client,
};
use axum::body::Bytes;

const IMAGE_BUCKET: &str = "flexforumimages1";

pub async fn s3_upload_image(
    s3_client: &Client,
    bytes: Bytes,
    username: &str,
    post_id: i64,
    image_num: usize,
) { // Result<PutObjectOutput, SdkError<PutObjectError,  Response>>
    let key = user_image_bucket_key(username, post_id, image_num);
    let sdk_body = SdkBody::from(bytes);
    let byte_stream = ByteStream::new(sdk_body);
    let res = s3_client
        .put_object()
        .bucket(IMAGE_BUCKET)
        .key(&key)
        .body(byte_stream)
        .send()
        .await;
    res
}

pub async fn s3_download_image(s3_client: &Client, username: &str, post_id: i64, image_num: usize) {
    let key = user_image_bucket_key(username, post_id, image_num);
    let res = s3_client
        .get_object()
        .bucket(IMAGE_BUCKET)
        .key(&key)
        .send()
        .await;
}

fn user_image_bucket_key(username: &str, post_id: i64, img_num: usize) -> String {
    format!("{}/{}/{}", username, post_id, img_num)
}
