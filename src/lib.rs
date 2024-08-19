pub mod mpv;
pub mod vrchat;

use std::path::PathBuf;

use anyhow::Result;
use lazy_regex::regex_replace_all;

/// # Errors
///
/// Will return `Err` if `std::fs::canonicalize` errors
///
/// # Panics
///
/// Will panic if an environment variable doesn't exist
pub fn parse_path_env(path: &str) -> Result<PathBuf> {
    let path = regex_replace_all!(r"(?:\$|%)(\w+)%?", path, |_, key| {
        std::env::var(key).unwrap_or_else(|_| panic!("Environment Variable not found: {key}"))
    });

    Ok(std::fs::canonicalize(path.as_ref())?)
}
