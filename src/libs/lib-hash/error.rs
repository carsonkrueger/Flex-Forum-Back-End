pub type HashResult<T> = Result<T, HashError>;

#[derive(Debug)]
pub enum HashError {
    Argon2Error(argon2::password_hash::Error),
}

impl From<argon2::password_hash::Error> for HashError {
    fn from(value: argon2::password_hash::Error) -> Self {
        HashError::Argon2Error(value)
    }
}
