//! This file contains miscellanous convenience functions

use super::launchermeta::get_version_manifest;
use crate::ferium_error::{FError, FResult};
use fancy_regex::Regex;
use shellexpand::tilde;
use std::env::consts::OS;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

/// Get a maximum of `count` number of the latest versions of Minecraft
///
/// Example:
/// ```rust
/// let latest_versions = get_latest_mc_versions(5);
/// println!("{:#?}", latest_versions);
/// ```
/// Output: (as of 08.2021)
/// ```text
/// [
///   1.17.1,
///   1.16.5,
///   1.15.2,
///   1.14.4,
///   1.13.2,
/// ]
/// ```
pub async fn get_latest_mc_versions(count: usize) -> FResult<Vec<String>> {
    let versions = get_version_manifest().await?.versions;
    let mut versions_to_display: Vec<String> = Vec::new();
    let mut major_versions_added: Vec<String> = Vec::new();
    // Remove minor version (e.g. "1.7.10" -> "1.7", "1.14" -> "1.14")
    let min_ver_remove = Regex::new(r"(?<=1.\d|\d\d)(\.\d{1,2}$)")?;

    for version in versions {
        if versions_to_display.len() > count {
            break;
        }

        // Remove minor version (e.g. "1.17.1" -> "1.17")
        let major_version = min_ver_remove.replace_all(&version.id, "").into();

        // If version is a release and it hasn't already been added
        if version.type_field == "release" && !major_versions_added.contains(&major_version) {
            versions_to_display.push(version.id);
            major_versions_added.push(major_version);
        }
    }

    Ok(versions_to_display)
}

/// Returns the default directory where mods are stored
pub fn get_mods_dir() -> FResult<PathBuf> {
    let home = tilde("~");
    let home = Path::new(home.as_ref());

    match OS {
        "macos" => Ok(home
            .join("Library")
            .join("ApplicationSupport")
            .join("minecraft")
            .join("mods")),
        "linux" => Ok(home.join(".minecraft").join("mods")),
        "windows" => Ok(home
            .join("AppData")
            .join("Roaming")
            .join(".minecraft")
            .join("mods")),
        _ => Err(FError::InvalidDeviceError),
    }
}

/// Run `print` macro and flush stdout to make results immediately appear
pub fn print(msg: impl std::fmt::Display) {
    print!("{}", msg);
    stdout().flush().unwrap();
}
