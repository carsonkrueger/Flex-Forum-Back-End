pub mod base;
pub mod content_model;
pub mod exercise_preset_model;
pub mod following_model;
pub mod interactions_matrix_model;
pub mod likes_model;
pub mod profile_picture_model;
pub mod seen_posts_model;
pub mod user_model;

pub type ModelResult<T> = std::result::Result<T, ModelError>;

#[derive(Debug)]
pub enum ModelError {
    // External errors
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for ModelError {
    fn from(value: sqlx::Error) -> Self {
        ModelError::Sqlx(value)
    }
}
