pub mod error;

use axum::body::Bytes;
use axum_typed_multipart::FieldData;
use error::{LibMultipartError, LibMultipartResult};

pub fn validate_content_type(
    field_data: &FieldData<Bytes>,
    content_types: &[&str],
) -> LibMultipartResult<()> {
    if let Some(s) = &field_data.metadata.content_type {
        for &c in content_types {
            if c == s.as_str() {
                return Ok(());
            }
        }
    }
    Err(LibMultipartError::InvalidContentType(
        field_data.metadata.name.clone().unwrap_or("".to_string()),
    ))
}
