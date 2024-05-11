pub mod base;
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
