//! This file contains miscellanous convenience functions

use super::launchermeta::get_version_manifest;
use crate::ferium_error::{FError, FResult};
use fancy_regex::Regex;
use shellexpand::tilde;
use std::env::consts::OS;
use std::path::{Path, PathBuf};

// Only macOS uses a sync file picker
#[cfg(target_os = "macos")]
/// Uses the appropriate file picker to pick a file
pub async fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

// Other OSs can use the async version
#[cfg(not(target_os = "macos"))]
/// Uses the appropriate file picker to pick a file
pub async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new().pick_folder().await
}

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

    for version in versions {
        if versions_to_display.len() > count {
            break;
        }

        let major_version = remove_minor_version(&version.id)?;

        // If version is a release and it hasn't already been added
        if version.type_field == "release" && !major_versions_added.contains(&major_version) {
            versions_to_display.push(version.id);
            major_versions_added.push(major_version);
        }
    }

    Ok(versions_to_display)
}

/// Removes the minor version from semver formatted strings
///
/// ```rust
/// assert_eq!(remove_minor_version("1.7.10"), "1.7");
/// assert_eq!(remove_minor_version("1.14.4"), "1.14");
/// // Versions already without a minor version are preserved
/// assert_eq!(remove_minor_version("1.14"), "1.14");
/// ```
pub fn remove_minor_version(string: &str) -> FResult<String> {
    let min_ver_remove = Regex::new(r"(?<=1.\d|\d\d)(\.\d{1,2}$)")?;
    Ok(min_ver_remove.replace_all(string, "").into())
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
