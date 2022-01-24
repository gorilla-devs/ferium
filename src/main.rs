mod util;

use ansi_term::Colour::{Green, White};
use clap::StructOpt;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use ferinth::Ferinth;
use furse::Furse;
use octocrab::Octocrab;
use std::{
	fs::{create_dir_all, OpenOptions},
	io::Write,
	path::PathBuf,
};
use util::{
	cli::{Ferium, ProfileSubCommands, SubCommands},
	ferium_error,
	ferium_error::{FError, FResult},
	json, launchermeta, wrappers,
};

#[tokio::main]
async fn main() {
	if let Err(err) = actual_main().await {
		// If an error occures, print the error message
		println!("{}", err);
		// And exit with an exit code
		std::process::exit(1);
	}
}

async fn actual_main() -> FResult<()> {
	// Get the command to execute from Clap
	// This also displays help page or version
	let cli_app = Ferium::parse();

	// Check for an internet connection
	if online::check(Some(1)).await.is_err() {
		// If it takes more than 1 second
		// show that we're checking the internet connection
		// and wait for 4 more seconds
		eprint!("Checking internet connection... ");
		match online::check(Some(4)).await {
			Ok(_) => println!("✓"),
			Err(_) => {
				return Err(FError::Quit(
					"× Ferium requires an internet connection to work",
				))
			},
		}
	};

	let github = octocrab::instance();
	let modrinth = Ferinth::new("ferium");
	let curseforge = Furse::new(env!(
		"CURSEFORGE_API_KEY",
		"A CurseForge API key is required to build. If you don't have one, you can bypass this by setting the variable to a blank string, however anything using the CurseForge API will not work."
	));
	// Ferium's config file
	let config_file = json::get_config_file()?;
	// Deserialise `config_file` to a config
	let mut config: json::Config = match serde_json::from_reader(&config_file) {
		Ok(config) => config,
		Err(err) => {
			return Err(FError::QuitFormatted(format!(
				"Error decoding configuration file, {} at {:?} {}:{}",
				// Error name
				FError::JSONError(err.classify()),
				// File path so that users can find it
				json::get_config_file_path(),
				// Location within config file
				err.line(),
				err.column()
			)));
		},
	};

	// The create command must run before getting the profile so that configs without profiles can have profiles added to them
	if let SubCommands::Profile {
		subcommand:
			ProfileSubCommands::Create {
				game_version,
				force_game_version,
				mod_loader,
				name,
				output_dir,
			},
	} = cli_app.subcommand
	{
		create(
			&mut config,
			game_version,
			force_game_version,
			mod_loader,
			name,
			output_dir,
		)
		.await?;

		// Update config file with new values
		json::write_to_config(config_file, &config)?;

		return Ok(());
	}

	// Get a mutable reference to the active profile
	let profile = if let Some(profile) = config.profiles.get_mut(config.active_profile) {
		profile
	} else {
		if config.profiles.is_empty() {
			return Err(FError::Quit (
				"There are no profiles configured. Add a profile to your config using `ferium profile create`"
			));
		}
		// Default to first profile if index is set incorrectly
		config.active_profile = 0;
		json::write_to_config(config_file, &config)?;
		return Err(FError::Quit(
			"Active profile index points to a non existent profile. Switched to first profile",
		));
	};

	// Run function(s) based on the sub(sub)command to be executed
	match cli_app.subcommand {
		SubCommands::AddModrinth { mod_id } => add_mod_modrinth(&modrinth, mod_id, profile).await?,
		SubCommands::AddGithub { owner, name } => {
			add_repo_github(&github, owner, name, profile).await?;
		},
		SubCommands::AddCurseforge { project_id } => {
			add_project_curseforge(&curseforge, project_id, profile).await?;
		},
		SubCommands::List { verbose } => {
			check_empty_profile(profile)?;
			list(&curseforge, &modrinth, &github, profile, verbose).await?;
		},
		SubCommands::Profile { subcommand } => match subcommand {
			ProfileSubCommands::Configure {
				game_version,
				mod_loader,
				name,
				output_dir,
			} => configure(profile, game_version, mod_loader, name, output_dir).await?,
			// This must have been checked earlier before getting the profile
			ProfileSubCommands::Create { .. } => unreachable!(),
			ProfileSubCommands::Delete { profile_name } => delete(&mut config, profile_name)?,
			ProfileSubCommands::List => list_profiles(&config),
		},
		SubCommands::Remove { mod_names } => {
			check_empty_profile(profile)?;
			remove(&curseforge, &modrinth, &github, profile, mod_names).await?;
		},
		SubCommands::Switch { profile_name } => switch(&mut config, profile_name)?,
		SubCommands::Upgrade {
			no_picker,
			no_patch_check,
		} => {
			check_empty_profile(profile)?;
			create_dir_all(&profile.output_dir)?;
			upgrade_curseforge(&curseforge, profile, no_patch_check).await?;
			upgrade_modrinth(&modrinth, profile, no_patch_check).await?;
			upgrade_github(&github, profile, no_picker).await?;
		},
	};

	// Update config file with new values
	json::write_to_config(config_file, &config)?;

	Ok(())
}

/// Check if `profile` is empty, and if so return an error
fn check_empty_profile(profile: &json::Profile) -> FResult<()> {
	if profile.github_repos.is_empty()
		&& profile.modrinth_mods.is_empty()
		&& profile.curse_projects.is_empty()
	{
		Err(FError::EmptyConfigFile)
	} else {
		Ok(())
	}
}

fn list_profiles(config: &json::Config) {
	for profile in &config.profiles {
		println!(
			"{}
		\r  Output directory:    {:?}
		\r  Minecraft Version:   {}
		\r  Mod Loader:          {}
		\r  CurseForge Projects: {}
		\r  Modrinth Mods:       {}
		\r  GitHub Repositories: {}\n",
			profile.name,
			profile.output_dir,
			profile.game_version,
			profile.mod_loader,
			profile.curse_projects.len(),
			profile.modrinth_mods.len(),
			profile.github_repos.len(),
		);
	}
}

async fn create(
	config: &mut json::Config,
	game_version: Option<String>,
	force_game_version: bool,
	mod_loader: Option<json::ModLoaders>,
	name: Option<String>,
	output_dir: Option<PathBuf>,
) -> FResult<()> {
	match (game_version, mod_loader, name, output_dir) {
		(Some(game_version), Some(mod_loader), Some(name), Some(output_dir)) => {
			// If force game version is false
			if !force_game_version {
				// And if the game_version provided does not exist
				if !launchermeta::get_version_manifest()
					.await?
					.versions
					.iter()
					.any(|version| version.id == game_version)
				{
					// Then error out
					return Err(FError::QuitFormatted(format!(
						"The game version {} does not exist",
						game_version
					)));
				}
			}
			// Check that there isn't already a profile with the same name
			for profile in &config.profiles {
				if profile.name == name {
					return Err(FError::QuitFormatted(format!(
						"A profile with name {} already exists", name.as_str()
					)));
				}
			}
			// Check that the output_dir isn't relative
			if !output_dir.is_absolute() {
				return Err(FError::Quit(
					"The provided output directory is not absolute, i.e. it is a relative path",
				));
			}
			config.profiles.push(json::Profile {
				name,
				output_dir,
				game_version,
				mod_loader,
				curse_projects: Vec::new(),
				modrinth_mods: Vec::new(),
				github_repos: Vec::new(),
			}); // Create profile
		},
		(None, None, None, None) => {
			println!("Please enter the details for the new profile");
			// Create profile using the UI
			config
				.profiles
				.push(json::Profile::create_ui(config).await?);
		},
		// Either all or none of these options should exist
		// TODO: make this into a group in the Clap app
		_ => {
			return Err(FError::Quit(
				"Provide all four arguments to create a profile using options",
			))
		},
	}

	config.active_profile = config.profiles.len() - 1; // Make created profile active
	Ok(())
}

fn delete(config: &mut json::Config, profile_name: Option<String>) -> FResult<()> {
	let selection = match profile_name {
		// If the profile name has been provided as an option
		Some(profile_name) => {
			// Sort profiles by their names
			config
				.profiles
				.sort_unstable_by_key(|profile| profile.name.clone());
			// Binary search the profile by their names
			match config
				.profiles
				.binary_search_by_key(&&profile_name, |profile| &profile.name)
			{
				// If the profile is found, return its index
				Ok(selection) => selection,
				// Else return an error
				Err(_) => return Err(FError::Quit("The profile name provided does not exist")),
			}
		},
		None => {
			let profile_names = config
				.profiles
				.iter()
				.map(|profile| &profile.name)
				.collect::<Vec<_>>();

			let selection = Select::with_theme(&ColorfulTheme::default())
				.with_prompt("Select which profile to delete")
				.items(&profile_names)
				.default(config.active_profile)
				.interact_opt()?;
			// Remove provided profile if one was selected
			if let Some(selection) = selection {
				selection
			} else {
				return Ok(());
			}
		},
	};
	// If the currently selected profile is being removed
	if config.active_profile == selection {
		// Default to the first profile
		config.active_profile = 0;
	}
	// Remove provided profile
	config.profiles.swap_remove(selection);
	Ok(())
}

fn switch(config: &mut json::Config, profile_name: Option<String>) -> FResult<()> {
	if config.profiles.len() < 2 {
		Err(FError::Quit("There is less than 2 profiles in your config"))
	} else if let Some(profile_name) = profile_name {
		// Sort profiles by name
		config
			.profiles
			.sort_unstable_by_key(|profile| profile.name.clone());
		// Binary search the profile
		match config
			.profiles
			.binary_search_by_key(&&profile_name, |profile| &profile.name)
		{
			Ok(selection) => {
				config.active_profile = selection;
				Ok(())
			},
			Err(_) => Err(FError::Quit("The profile provided does not exist")),
		}
	} else {
		let profile_names = config
			.profiles
			.iter()
			.map(|profile| &profile.name)
			.collect::<Vec<_>>();

		let selection = Select::with_theme(&ColorfulTheme::default())
			.with_prompt("Select which profile to switch to")
			.items(&profile_names)
			.default(config.active_profile)
			.interact()?;
		config.active_profile = selection;
		Ok(())
	}
}

async fn configure(
	profile: &mut json::Profile,
	game_version: Option<String>,
	mod_loader: Option<json::ModLoaders>,
	name: Option<String>,
	output_dir: Option<PathBuf>,
) -> FResult<()> {
	let mut interactive = true;

	if let Some(game_version) = game_version {
		profile.game_version = game_version;
		interactive = false;
	}
	if let Some(mod_loader) = mod_loader {
		profile.mod_loader = mod_loader;
		interactive = false;
	}
	if let Some(name) = name {
		profile.name = name;
		interactive = false;
	}
	if let Some(output_dir) = output_dir {
		profile.output_dir = output_dir;
		interactive = false;
	}

	if interactive {
		let items = vec![
			// Show a file dialog
			"Mods output directory",
			// Show a picker of Minecraft versions to select from
			"Minecraft version",
			// Show a picker to change mod loader
			"Mod loader",
			// Show a dialog to change name
			"Profile Name",
			// Quit the configuration
			"Quit",
		];

		loop {
			let selection = Select::with_theme(&ColorfulTheme::default())
				.with_prompt("Which setting would you like to change")
				.items(&items)
				.interact_opt()?;

			if let Some(index) = selection {
				match index {
					0 => {
						eprint!(
							"{} {}",
							Green.paint("✔"),
							White.bold().paint("Pick a mod output directory   "),
						);
						// Let user pick output directory
						if let Some(dir) = wrappers::pick_folder(&profile.output_dir).await {
							profile.output_dir = dir;
						}
						println!(
							"{}",
							Green.paint(profile.output_dir.to_str().ok_or(FError::OptionError)?)
						);
					},
					1 => {
						// Let user pick mc version from latest 10 versions
						let mut versions = wrappers::get_latest_mc_versions(10).await?;
						let index = Select::with_theme(&ColorfulTheme::default())
							.with_prompt("Select a Minecraft version")
							.items(&versions)
							.default(0)
							.interact_opt()?;
						if let Some(i) = index {
							profile.game_version = versions.swap_remove(i);
						}
					},
					2 => {
						// Let user pick mod loader
						let mod_loaders = ["Fabric", "Forge"];
						let index = Select::with_theme(&ColorfulTheme::default())
							.with_prompt("Pick a mod loader")
							.items(&mod_loaders)
							.default(match profile.mod_loader {
								json::ModLoaders::Fabric => 0,
								json::ModLoaders::Forge => 1,
							})
							.interact_opt()?;
						if index == Some(0) {
							profile.mod_loader = json::ModLoaders::Fabric;
						} else if index == Some(1) {
							profile.mod_loader = json::ModLoaders::Forge;
						}
					},
					3 => {
						let name = Input::with_theme(&ColorfulTheme::default())
							.with_prompt("Change the profile's name")
							.default(profile.name.clone())
							.interact_text()?;
						profile.name = name;
					},
					4 => break,
					_ => unreachable!(),
				}
				println!();
			} else {
				break;
			}
		}
	}

	Ok(())
}

/// Display a list of mods and repos in the profile to select from and remove selected ones
async fn remove(
	curseforge: &Furse,
	modrinth: &Ferinth,
	github: &Octocrab,
	profile: &mut json::Profile,
	mod_names: Option<Vec<String>>,
) -> FResult<()> {
	let mut names: Vec<String> = Vec::new();

	// Get the names of the mods
	eprint!("Gathering mod names... ");
	for mod_id in &profile.modrinth_mods {
		let mod_ = modrinth.get_mod(mod_id).await?;
		names.push(mod_.title);
	}

	for repo_name in &profile.github_repos {
		let repo = github.repos(&repo_name.0, &repo_name.1).get().await?;
		names.push(repo.name);
	}

	for project_id in &profile.curse_projects {
		let project = curseforge.get_mod(*project_id).await?;
		names.push(project.name);
	}
	println!("✓");

	let mut items_to_remove = Vec::new();
	match mod_names {
		Some(mod_names) => {
			// Here we use inefficient double nested for loops because
			// 1. We need to retain the indices for removal so we cannot binary search
			// 2. We want to remove duplicates too
			// 3. We want to use the same items_to_remove format so that both options and user selected data can be processed using the same algorithm

			// For each mod name to remove
			for mod_name in mod_names {
				let mut found_mod = false;
				// Search through all the mod names
				for (i, name) in names.iter().enumerate() {
					// If a match is found, push the match's index to items_to_remove
					if name.to_lowercase().contains(&mod_name.to_lowercase()) {
						found_mod = true;
						items_to_remove.push(i);
					}
				}

				// If a mod is not found, throw an error
				if !found_mod {
					return Err(FError::QuitFormatted(format!(
						"A mod called {} is not present in this profile",
						mod_name
					)));
				}
			}
		},
		None => {
			// Show selection menu
			items_to_remove = match MultiSelect::with_theme(&ColorfulTheme::default())
				.with_prompt("Select mods and/or repositories to remove")
				.items(&names)
				.interact_opt()?
			{
				Some(items_to_remove) => items_to_remove,
				None => return Ok(()), // Exit if the user cancelled
			};
		},
	}

	// Sort the indices in ascending order to fix moving indices during removal
	items_to_remove.sort_unstable();
	items_to_remove.reverse();

	// For each mod to remove
	for index in items_to_remove {
		// If index is larger than the length of the repos mod_ids, then the index is for curse projects
		if index >= (profile.modrinth_mods.len() + profile.github_repos.len()) {
			// Offset the index by the proper amount
			let index = index - (profile.modrinth_mods.len() + profile.github_repos.len());

			// Remove item from profile's curse projects
			profile.curse_projects.swap_remove(index);
		}
		// If index is larger than the mod_ids' length, then the index is for repos
		else if index >= profile.modrinth_mods.len() {
			// Offset the index by the proper amount
			let index = index - profile.modrinth_mods.len();

			// Remove item from profile's repos
			profile.github_repos.swap_remove(index);
		// Or else its for the mod_ids
		} else {
			// Remove item from profile' mod ids
			profile.modrinth_mods.swap_remove(index);
		}
	}

	Ok(())
}

/// Check if repo `owner`/`repo_name` exists and releases mods, and if so add repo to `profile`
async fn add_repo_github(
	github: &Octocrab,
	owner: String,
	repo_name: String,
	profile: &mut json::Profile,
) -> FResult<()> {
	eprint!("Adding GitHub repository... ");

	// Get repository and releases data
	let repo_handler = github.repos(owner, repo_name);
	let repo = repo_handler.get().await?;
	// Get the name of the repository as a tuple
	let repo_name_split = repo
		.full_name
		.as_ref()
		.ok_or(FError::OptionError)?
		.split('/')
		.collect::<Vec<_>>();
	let repo_name = (repo_name_split[0].into(), repo_name_split[1].into());

	// Check if repo has already been added
	if profile.github_repos.contains(&repo_name) {
		return Err(FError::Quit("× Repository already added to profile!"));
	}

	let releases = repo_handler.releases().list().send().await?;
	let mut contains_jar_asset = false;

	// Search every asset to check if the releases contain JAR files (a mod file)
	'outer: for release in releases {
		for asset in release.assets {
			if asset.name.contains("jar") {
				// If JAR release is found, set flag to true and break
				contains_jar_asset = true;
				break 'outer;
			}
		}
	}

	if contains_jar_asset {
		// Append repo to profile
		profile.github_repos.push(repo_name);
		println!("✓");
	} else {
		return Err(FError::Quit("× Repository does not release mods!"));
	}

	Ok(())
}

/// Check if mod with ID `mod_id` exists, if so add that mod to `profile`
async fn add_mod_modrinth(
	modrinth: &Ferinth,
	mod_id: String,
	profile: &mut json::Profile,
) -> FResult<()> {
	eprint!("Adding Modrinth mod... ");

	// Check if mod exists
	match modrinth.get_mod(&mod_id).await {
		Ok(mod_) => {
			// Check if mod has already been added
			if profile.modrinth_mods.contains(&mod_.id) {
				return Err(FError::Quit("× Mod already added to profile!"));
			}
			// And if it hasn't, append mod to profile and write
			profile.modrinth_mods.push(mod_.id);
			println!("✓ ({})", mod_.title);

			Ok(())
		},
		Err(_) => {
			// Else return an error
			Err(FError::QuitFormatted(format!(
				"× Mod with ID `{}` does not exist!",
				mod_id
			)))
		},
	}
}

async fn add_project_curseforge(
	curseforge: &Furse,
	project_id: i32,
	profile: &mut json::Profile,
) -> FResult<()> {
	eprint!("Adding CurseForge mod... ");

	// Check if project exists
	match curseforge.get_mod(project_id).await {
		Ok(project) => {
			if profile.curse_projects.contains(&project.id) {
				Err(FError::Quit("× Project already added to profile!"))
			} else {
				profile.curse_projects.push(project.id);
				println!("✓ ({})", project.name);
				Ok(())
			}
		},
		Err(err) => Err(FError::QuitFormatted(format!(
			"× Project with ID `{}` does not exist! ({})",
			project_id, err
		))),
	}
}

/// List all the mods in `profile` with some of their metadata
async fn list(
	curseforge: &Furse,
	modrinth: &Ferinth,
	github: &Octocrab,
	profile: &json::Profile,
	verbose: bool,
) -> FResult<()> {
	for project_id in &profile.curse_projects {
		let project = curseforge.get_mod(*project_id).await?;
		let mut authors = String::new();
		for author in project.authors {
			authors.push_str(&author.name);
			authors.push_str(", ");
		}
		// Trim trailing ', '
		authors.truncate(authors.len() - 2);

		let mut categories = String::new();
		for category in project.categories {
			categories.push_str(&category.name);
			categories.push_str(", ");
		}
		// Trim trailing ', '
		categories.truncate(categories.len() - 2);

		println!(
			"{}
			\r  {}\n
			\r  Link:        {}
			\r  Source:      CurseForge Project
			\r  Open Source: {}
			\r  Downloads:   {}
			\r  Authors:     {}
			\r  Category:    {}\n",
			project.name,
			project.summary,
			project.links.website_url,
			project
				.links
				.source_url
				.map_or("No".into(), |url| format!("Yes ({})", url)),
			project.download_count,
			authors,
			categories,
		);
	}

	for mod_id in &profile.modrinth_mods {
		// Get mod metadata
		let mod_ = modrinth.get_mod(mod_id).await?;
		if verbose {
			let team_members = modrinth.list_team_members(&mod_.team).await?;

			// Get the usernames of all the developers
			let mut developers = String::new();
			for member in team_members {
				let user = modrinth.get_user(&member.user_id).await?;
				developers.push_str(&user.username);
				developers.push_str(", ");
			}
			// Trim trailing ', '
			developers.truncate(developers.len() - 2);

			println!(
				"{}
            \r  {}\n
            \r  Link:           https://modrinth.com/mod/{}
            \r  Source:         Modrinth Mod
            \r  Open Source:    {}
            \r  Downloads:      {}
            \r  Developers:     {}
            \r  Client side:    {}
            \r  Server side:    {}
            \r  License:        {}{}\n",
				mod_.title,
				mod_.description,
				mod_.slug,
				mod_.source_url
					.map_or("No".into(), |url| { format!("Yes ({})", url) }),
				mod_.downloads,
				developers,
				mod_.client_side,
				mod_.server_side,
				mod_.license.name,
				mod_.license
					.url
					.map_or("".into(), |url| { format!(" ({})", url) }),
			);
		} else {
			println!(
				"{}
                \r  {}\n
                \r  Link:     https://modrinth.com/mod/{}
                \r  Source:   Modrinth Mod
                \r  Code:     {}
                \r  License:  {}\n",
				mod_.title,
				mod_.description,
				mod_.slug,
				mod_.source_url
					.map_or("Closed source".into(), |url| { url }),
				mod_.license.name
			);
		}
	}

	for repo_name in &profile.github_repos {
		// Get repository metadata
		let repo_handler = github.repos(&repo_name.0, &repo_name.1);
		let repo = repo_handler.get().await?;
		if verbose {
			let releases = repo_handler.releases().list().send().await?;
			let mut downloads = 0;

			// Calculate number of downloads
			for release in releases {
				for asset in release.assets {
					downloads += asset.download_count;
				}
			}

			// Print repository data formatted
			println!(
				"{}{}\n
            \r  Link:           {}
            \r  Source:         GitHub Repository
            \r  Downloads:      {}
            \r  Developer:      {}{}\n",
				repo.name,
				repo.description
					.map_or("".into(), |description| { format!("\n  {}", description) }),
				repo.html_url.ok_or(FError::OptionError)?,
				downloads,
				repo.owner.ok_or(FError::OptionError)?.login,
				if let Some(license) = repo.license {
					format!(
						"\n  License:        {}{}",
						license.name,
						license
							.html_url
							.map_or("".into(), |url| { format!(" ({})", url) })
					)
				} else {
					"".into()
				},
			);
		} else {
			println!(
				"{}{}\n
                \r  Link:     {}
                \r  Source:   GitHub Repository{}\n",
				repo.name,
				repo.description
					.map_or("".into(), |description| { format!("\n  {}", description) }),
				repo.html_url.ok_or(FError::OptionError)?,
				if let Some(license) = repo.license {
					format!("\n  License:  {}", license.name)
				} else {
					"".into()
				},
			);
		}
	}

	Ok(())
}

async fn upgrade_curseforge(
	curseforge: &Furse,
	profile: &json::Profile,
	no_patch_check: bool,
) -> FResult<()> {
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
		let game_version_to_check = wrappers::remove_semver_patch(&profile.game_version)?;

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
					.open(profile.output_dir.join(&latest_compatible_file.file_name))?;

				let file_contents = curseforge
					.download_mod_file_from_file(latest_compatible_file)
					.await?;

				mod_file.write_all(&file_contents)?;
				println!("✓\n");
			},
			None => {
				return Err(FError::QuitFormatted(format!(
					"× No version of {} is compatible for {} {}",
					project.name, profile.mod_loader, profile.game_version,
				)));
			},
		}
	}

	Ok(())
}

/// Download and install all the GitHub mods in `profile`
async fn upgrade_github(
	github: &Octocrab,
	profile: &json::Profile,
	no_picker: bool,
) -> FResult<()> {
	for repo_name in &profile.github_repos {
		println!("Downloading {}", repo_name.1);
		eprint!("  [1] Getting release information... ");

		let repo_handler = github.repos(&repo_name.0, &repo_name.1);
		let releases = repo_handler.releases().list().send().await?;
		let version_to_check = wrappers::remove_semver_patch(&profile.game_version)?;

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
                        release.body.as_ref().ok_or(FError::OptionError)?.contains(&version_to_check)
                        // the asset's name
                        || asset.name.contains(&version_to_check)
						// and even the release name
                        || release.name.as_ref().ok_or(FError::OptionError)?.contains(&version_to_check))
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
			return Err(FError::Quit(
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
			.open(profile.output_dir.join(&asset_to_download.name))?;

		// Write download to mod JAR file
		mod_file.write_all(&contents)?;
		println!("✓\n");
	}

	Ok(())
}

/// Download and install all Modrinth mods in `profile`
async fn upgrade_modrinth(
	modrinth: &Ferinth,
	profile: &json::Profile,
	no_patch_check: bool,
) -> FResult<()> {
	for mod_id in &profile.modrinth_mods {
		// Get mod metadata
		let mod_ = modrinth.get_mod(mod_id).await?;
		println!("Downloading {}", mod_.title);

		eprint!("  [1] Getting version information... ");
		// Get the versions of the mod
		let versions = modrinth.list_versions(&mod_.id).await?;
		let game_version_to_check = wrappers::remove_semver_patch(&profile.game_version)?;

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
				return Err(FError::QuitFormatted(format!(
					"× No version of {} is compatible for {} {}",
					mod_.title, profile.mod_loader, profile.game_version,
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
			.open(profile.output_dir.join(&latest_version.files[0].filename))?;

		// Write contents to JAR file
		mod_file.write_all(&contents)?;
		println!("✓\n");
	}

	Ok(())
}
