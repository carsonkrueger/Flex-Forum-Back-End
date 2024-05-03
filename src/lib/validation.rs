use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z '-]+").unwrap());
