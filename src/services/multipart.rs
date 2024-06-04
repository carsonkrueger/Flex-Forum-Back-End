use std::io::Write;

use axum::body::Bytes;
use axum_typed_multipart::FieldData;

use crate::routes::{RouteError, RouterResult};

pub fn create_file(field_data: &FieldData<Bytes>, file_path: String) -> RouterResult<()> {
    let mut file = std::fs::File::options()
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|e| e)?;
    file.write_all(&field_data.contents).unwrap();
    Ok(())
}

pub fn validate_content_type(
    field_data: &FieldData<Bytes>,
    content_types: &[&str],
) -> RouterResult<()> {
    if let Some(s) = &field_data.metadata.content_type {
        for &c in content_types {
            if c == s.as_str() {
                return Ok(());
            }
        }
    }
    Err(RouteError::InvalidContentType(
        field_data.metadata.name.clone().unwrap_or("".to_string()),
    ))
}
