use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct LoginModel {
    #[validate(length(min = 1, max = 32, message = "Invalid username length"))]
    pub email: String,
    #[validate(length(min = 1, max = 32, message = "Invalid password length"))]
    pub password: String,
}
