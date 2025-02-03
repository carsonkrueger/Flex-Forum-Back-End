pub type LibMultipartResult<T> = Result<T, LibMultipartError>;

#[derive(Debug, Clone)]
pub enum LibMultipartError {
    InvalidContentType(String),
}
