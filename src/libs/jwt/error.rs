pub type JWTResult<T> = Result<T, JWTError>;

#[derive(Debug, Clone)]
pub enum JWTError {
    InvalidJWT,
    HashError,
    MissingJWTSignature,
    ExpiredJWT,
}

impl From<lib_hash::error::HashError> for JWTError {
    fn from(_: lib_hash::error::HashError) -> Self {
        Self::HashError
    }
}

impl From<chrono::ParseError> for JWTError {
    fn from(_: chrono::ParseError) -> Self {
        Self::InvalidJWT
    }
}
