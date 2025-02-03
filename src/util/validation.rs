use once_cell::sync::Lazy;
use regex::Regex;
use validator::Validate;

use crate::route::error::{RouteError, RouteResult};

pub static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z '-]+").unwrap());
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-Za-z_-]+").unwrap());

pub fn validate_struct(item: &impl Validate) -> RouteResult<()> {
    item.validate()
        .map_err(|e| RouteError::Validation(e.to_string()))
}
