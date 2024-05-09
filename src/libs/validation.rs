use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z '-]+").unwrap());
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-Za-z_-]+").unwrap());
