use crate::routes::{Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use validator::Validate;

pub static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z '-]+").unwrap());
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-Za-z_-]+").unwrap());

pub fn validate(item: impl Validate) -> Result<()> {
    item.validate()
        .map_err(|e| Error::Validation(e.to_string()))
}
