//! Contains miscellanous convenience functions

use super::launchermeta::{get_version_manifest, VersionType};
use crate::ferium_error::{FError, FResult};
use onig::Regex;
use shellexpand::tilde;
use std::{
	env::consts::OS,
	path::{Path, PathBuf},
};

// macOS can only use a sync file picker
#[cfg(target_os = "macos")]
#[allow(clippy::unused_async)]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &Path) -> Option<PathBuf> {
	rfd::FileDialog::new().set_directory(path).pick_folder()
}

// Other OSs can use the async version
#[cfg(not(target_os = "macos"))]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &Path) -> Option<PathBuf> {
	rfd::AsyncFileDialog::new()
		.set_directory(path)
		.pick_folder()
		.await
		.map(|handle| handle.path().into())
}

/// Get a maximum of `count` number of the latest versions of Minecraft
///
/// Example:
/// ```rust
/// assert_eq!(
///     get_latest_mc_versions(6),
///     // This will change as Minecraft updates get released
///     vec!([
///         "1.18.1".into()
///         "1.17.1".into(),
///         "1.16.5".into(),
///         "1.15.2".into(),
///         "1.14.4".into(),
///         "1.13.2".into(),
///     ])
/// );
/// ```
pub async fn get_latest_mc_versions(count: usize) -> FResult<Vec<String>> {
	let versions = get_version_manifest().await?.versions;
	let mut versions_to_display: Vec<String> = Vec::new();
	let mut major_versions_added: Vec<String> = Vec::new();

	for version in versions {
		if versions_to_display.len() >= count {
			break;
		}

		let major_version = remove_semver_patch(&version.id)?;

		// If version is a release and it hasn't already been added
		if matches!(version.type_field, VersionType::Release)
			&& !major_versions_added.contains(&major_version)
		{
			versions_to_display.push(version.id);
			major_versions_added.push(major_version);
		}
	}

	Ok(versions_to_display)
}

/// Removes the patch segment from semver formatted strings using a regex
///
/// ```rust
/// assert_eq!(remove_semver_patch("1.7.10")?, "1.7".into());
/// assert_eq!(remove_semver_patch("1.14.4")?, "1.14".into());
/// // Versions already without a minor version are preserved
/// assert_eq!(remove_semver_patch("1.18")?, "1.18".into());
/// ```
pub fn remove_semver_patch(semver: &str) -> FResult<String> {
	let semver_patch_remove = Regex::new(r"(?<=\d{1,}\.\d{1,})(\.\d{1,}$)")?;
	Ok(semver_patch_remove.replace_all(semver, ""))
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
