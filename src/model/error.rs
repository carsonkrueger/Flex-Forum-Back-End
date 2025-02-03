pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug)]
pub enum ModelError {
    Sqlx(sqlx::Error),
    SeaQuery(sea_query::error::Error),
}

impl ToString for ModelError {
    fn to_string(&self) -> String {
        match self {
            Self::SeaQuery(e) => e.to_string(),
            Self::Sqlx(e) => e.to_string(),
        }
    }
}

impl From<sqlx::Error> for ModelError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl From<sea_query::error::Error> for ModelError {
    fn from(value: sea_query::error::Error) -> Self {
        Self::SeaQuery(value)
    }
}
