use crate::error::{Error, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use ferinth::Ferinth;
use furse::Furse;
use libium::{config, misc};
use octocrab::Octocrab;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

pub async fn curseforge(
	curseforge: &Furse,
	profile: &config::structs::Profile,
	no_patch_check: bool,
) -> Result<()> {
	for project_id in &profile.curse_projects {
		let project = curseforge.get_mod(*project_id).await?;
		println!("Downloading {}", project.name);
		eprint!("  [1] Getting file information... ");

		// Get the project's files
		let mut files = curseforge.get_mod_files(*project_id).await?;
		// Sorting in chronological order using file IDs
		files.sort_by_key(|file| file.id);
		// Reverse so that the newest files come first
		files.reverse();

		let mut latest_compatible_file = None;
		let game_version_to_check = misc::remove_semver_patch(&profile.game_version)?;

		for file in &files {
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

		match latest_compatible_file {
			Some(latest_compatible_file) => {
				println!("✓");

				eprint!(
					"  [2] Downloading {}... ",
					latest_compatible_file.display_name
				);

				let mut mod_file = OpenOptions::new()
					.read(true)
					.write(true)
					.truncate(true)
					.create(true)
					.open(profile.output_dir.join(&latest_compatible_file.file_name))
					.await?;

				let file_contents = curseforge
					.download_mod_file_from_file(latest_compatible_file)
					.await?;

				mod_file.write_all(&file_contents).await?;
				println!("✓\n");
			},
			None => {
				return Err(Error::QuitFormatted(format!(
					"× No version of {} is compatible for {} {}",
					project.name, profile.mod_loader, profile.game_version,
				)));
			},
		}
	}

	Ok(())
}

/// Download and install all the GitHub mods in `profile`
pub async fn github(
	github: &Octocrab,
	profile: &config::structs::Profile,
	no_picker: bool,
) -> Result<()> {
	for repo_name in &profile.github_repos {
		println!("Downloading {}", repo_name.1);
		eprint!("  [1] Getting release information... ");

		let repo_handler = github.repos(&repo_name.0, &repo_name.1);
		let releases = repo_handler.releases().list().send().await?;
		let version_to_check = misc::remove_semver_patch(&profile.game_version)?;

		// A vector of assets that are compatible with the game version and mod loader
		let mut asset_candidates = Vec::new();
		// Whether the mod specifies the mod loader in its assets' names
		let mut specifies_loader = false;

		// Try to get the compatible assets
		for release in &releases {
			for asset in &release.assets {
				// If the asset specifies the mod loader, set the `specifies_loader` flag to true
				if asset.name.to_lowercase().contains("fabric")
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
                        // Check the release body
                        release.body.as_ref().ok_or(Error::OptionError)?.contains(&version_to_check)
                        // the asset's name
                        || asset.name.contains(&version_to_check)
						// and even the release name
                        || release.name.as_ref().ok_or(Error::OptionError)?.contains(&version_to_check))
                    // Check if its a JAR file
                    && asset.name.contains("jar")
				{
					// Specify this asset as a compatible asset
					asset_candidates.push(asset);
				}
			}
		}

		// If 1 compatible asset was found, use it
		let asset_to_download = if asset_candidates.len() == 1 {
			println!("✓");
			asset_candidates[0]
		// If none were found, error out
		} else if asset_candidates.is_empty() {
			return Err(Error::Quit(
				"× Could not find a compatible asset to download",
			));
		// If more than 1 was found, let the user select which one to use
		} else {
			println!("✓");
			if no_picker {
				asset_candidates[0]
			} else {
				let mut asset_candidate_names = Vec::new();
				for asset_candidate in &asset_candidates {
					asset_candidate_names.push(&asset_candidate.name);
				}
				let selection = Select::with_theme(&ColorfulTheme::default())
					.with_prompt("Select the asset to download")
					.items(&asset_candidate_names)
					.interact()?;
				asset_candidates[selection]
			}
		};

		eprint!("  [2] Downloading {}... ", asset_to_download.name);

		// Get file contents
		let contents = reqwest::get(asset_to_download.browser_download_url.clone())
			.await?
			.bytes()
			.await?;

		// Open the mod JAR file
		let mut mod_file = OpenOptions::new()
			.read(true)
			.write(true)
			.truncate(true)
			.create(true)
			.open(profile.output_dir.join(&asset_to_download.name))
			.await?;

		// Write download to mod JAR file
		mod_file.write_all(&contents).await?;
		println!("✓\n");
	}

	Ok(())
}

/// Download and install all Modrinth mods in `profile`
pub async fn modrinth(
	modrinth: &Ferinth,
	profile: &config::structs::Profile,
	no_patch_check: bool,
) -> Result<()> {
	for project_id in &profile.modrinth_mods {
		// Get mod metadata
		let project = modrinth.get_project(project_id).await?;
		println!("Downloading {}", project.title);

		eprint!("  [1] Getting version information... ");
		// Get the versions of the mod
		let versions = modrinth.list_versions(&project.id).await?;
		let game_version_to_check = misc::remove_semver_patch(&profile.game_version)?;

		let mut latest_compatible_version = None;

		// Check if a version compatible with the game version and mod loader specified in the profile is available
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

		let latest_version = match latest_compatible_version {
			Some(version) => version,
			// If version compatible with game version does not exist, throw an error
			None => {
				return Err(Error::QuitFormatted(format!(
					"× No version of {} is compatible for {} {}",
					project.title, profile.mod_loader, profile.game_version,
				)));
			},
		};

		println!("✓");

		eprint!("  [2] Downloading {}... ", latest_version.name);

		// Get file contents
		let contents = modrinth
			.download_version_file(&latest_version.files[0])
			.await?;

		// Open mod JAR file
		let mut mod_file = OpenOptions::new()
			.read(true)
			.write(true)
			.truncate(true)
			.create(true)
			.open(profile.output_dir.join(&latest_version.files[0].filename))
			.await?;

		// Write contents to JAR file
		mod_file.write_all(&contents).await?;
		println!("✓\n");
	}

	Ok(())
}
