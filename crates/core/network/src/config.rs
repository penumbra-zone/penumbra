//! Utilities for managing local directories containing node state.
use directories::UserDirs;

use regex::{Captures, Regex};

use std::{env::current_dir, path::PathBuf};
use url::Url;

/// Expand tildes in a path.
/// Modified from `<https://stackoverflow.com/a/68233480>`
pub fn canonicalize_path(input: &str) -> PathBuf {
    let tilde = Regex::new(r"^~(/|$)").expect("tilde regex is valid");
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is
        input.into()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory
        PathBuf::from(&*tilde.replace(input, |c: &Captures| {
            if let Some(user_dirs) = UserDirs::new() {
                format!("{}{}", user_dirs.home_dir().to_string_lossy(), &c[1],)
            } else {
                c[0].to_string()
            }
        }))
    } else {
        PathBuf::from(format!(
            "{}/{}",
            current_dir()
                .expect("current working dir is valid")
                .display(),
            input
        ))
    }
}

/// Convert an optional CLI arg into a [`PathBuf`], defaulting to
/// `~/.penumbra/testnet_data`.
pub fn get_testnet_dir(testnet_dir: Option<PathBuf>) -> PathBuf {
    // By default output directory will be in `~/.penumbra/testnet_data/`
    match testnet_dir {
        Some(o) => o,
        None => canonicalize_path("~/.penumbra/testnet_data"),
    }
}

/// Check that a [Url] has all the necessary parts defined for use as a CLI arg.
pub fn url_has_necessary_parts(url: &Url) -> bool {
    url.scheme() != "" && url.has_host() && url.port().is_some()
}
