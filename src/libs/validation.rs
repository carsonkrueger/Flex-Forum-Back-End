use lib_routes::error::{RouteError, RouterResult};
use once_cell::sync::Lazy;
use regex::Regex;
use validator::Validate;

pub static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z '-]+").unwrap());
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-Za-z_-]+").unwrap());

pub fn validate_struct(item: &impl Validate) -> RouterResult<()> {
    item.validate()
        .map_err(|e| RouteError::Validation(e.to_string()))
}
