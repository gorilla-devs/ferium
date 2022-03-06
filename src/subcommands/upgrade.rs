use crate::error::{Error, Result};
use ferinth::Ferinth;
use furse::Furse;
use libium::{config, misc};
use octocrab::repos::RepoHandler;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

/// Write `contents` to a file with path `profile.output_dir`/`name`
async fn write_mod_file(
	profile: &config::structs::Profile,
	contents: bytes::Bytes,
	name: &str,
) -> Result<()> {
	// Open the mod JAR file
	let mut mod_file = OpenOptions::new()
		.read(true)
		.write(true)
		.truncate(true)
		.create(true)
		.open(profile.output_dir.join(name))
		.await?;

	// Write downloaded contents to mod JAR file
	mod_file.write_all(&contents).await?;
	Ok(())
}

/// Download and install the latest file of `project_id`
pub async fn curseforge(
	curseforge: &Furse,
	profile: &config::structs::Profile,
	project_id: i32,
	no_patch_check: bool,
) -> Result<()> {
	// Get the project's files
	let mut files = curseforge.get_mod_files(project_id).await?;
	// Sorting in chronological order
	files.sort_unstable_by_key(|file| file.file_date);
	// Reverse so that the newest files come first
	files.reverse();

	let mut latest_compatible_file = None;
	let game_version_to_check = misc::remove_semver_patch(&profile.game_version)?;

	for file in files {
		if no_patch_check {
			// Search every version to see if it contains the game version specified without patch
			if file
				.game_versions
				.iter()
				.any(|game_version| game_version.contains(&game_version_to_check))
				&& file.game_versions.contains(&profile.mod_loader.to_string())
			{
				latest_compatible_file = Some(file);
				break;
			}
		} else {
			// Or else just check if it contains the full version
			if file.game_versions.contains(&profile.game_version)
				&& file.game_versions.contains(&profile.mod_loader.to_string())
			{
				latest_compatible_file = Some(file);
				break;
			}
		}
	}

	if let Some(file) = latest_compatible_file {
		let contents = curseforge.download_mod_file_from_file(&file).await?;
		write_mod_file(profile, contents, &file.file_name).await?;
		Ok(())
	} else {
		Err(Error::Quit("Could not find a compatible file to download"))
	}
}

/// Download and install the latest release of `repo_handler`
pub async fn github(
	repo_handler: &RepoHandler<'_>,
	profile: &config::structs::Profile,
) -> Result<()> {
	let releases = repo_handler.releases().list().send().await?;
	let version_to_check = misc::remove_semver_patch(&profile.game_version)?;

	let mut asset_to_download = None;
	// Whether the mod loader is specified in asset names
	let mut specifies_loader = false;

	'outer: for release in &releases {
		for asset in &release.assets {
			// If the asset specifies the mod loader, set the `specifies_loader` flag to true
			// If it was already set, this is skipped
			if !specifies_loader && asset.name.to_lowercase().contains("fabric")
				|| asset.name.to_lowercase().contains("forge")
			{
				specifies_loader = true;
			}

			// If the mod loader is not specified then skip checking for the mod loader
			if (!specifies_loader
					// If it does specify, then check the mod loader
					|| asset.name.to_lowercase().contains(&profile.mod_loader.to_string().to_lowercase()))
                    // Check if the game version is compatible
                    && (
                        // Check the asset's name
                        asset.name.contains(&version_to_check)
						// and the release name
                        || release.name.as_ref().ok_or(Error::OptionError)?.contains(&version_to_check))
                    // Check if its a JAR file
                    && asset.name.contains("jar")
			{
				// Specify this asset as a compatible asset
				asset_to_download = Some(asset);
				break 'outer;
			}
		}
	}

	if let Some(asset_to_download) = asset_to_download {
		let contents = reqwest::get(asset_to_download.browser_download_url.clone())
			.await?
			.bytes()
			.await?;
		write_mod_file(profile, contents, &asset_to_download.name).await?;
		Ok(())
	} else {
		Err(Error::Quit("Could not find a compatible asset to download"))
	}
}

/// Download and install all Modrinth mods in `profile`
pub async fn modrinth(
	modrinth: &Ferinth,
	profile: &config::structs::Profile,
	project_id: &str,
	no_patch_check: bool,
) -> Result<()> {
	let project = modrinth.get_project(project_id).await?;

	// Get the versions of the mod
	let versions = modrinth.list_versions(&project.id).await?;

	let mut latest_compatible_version = None;
	let game_version_to_check = misc::remove_semver_patch(&profile.game_version)?;

	for version in versions {
		if no_patch_check {
			// Search every version to see if it contains the game version specified without patch
			if version
				.game_versions
				.iter()
				.any(|game_version| game_version.contains(&game_version_to_check))
				&& version
					.loaders
					.contains(&profile.mod_loader.to_string().to_lowercase())
			{
				latest_compatible_version = Some(version);
				break;
			}
		} else {
			// Or else just check if it contains the full version
			if version.game_versions.contains(&profile.game_version)
				&& version
					.loaders
					.contains(&profile.mod_loader.to_string().to_lowercase())
			{
				latest_compatible_version = Some(version);
				break;
			}
		}
	}

	if let Some(version) = latest_compatible_version {
		// Get file contents
		let contents = modrinth.download_version_file(&version.files[0]).await?;
		write_mod_file(profile, contents, &version.files[0].filename).await?;
		Ok(())
	} else {
		Err(Error::Quit("Could not find a compatible file to download"))
	}
}
