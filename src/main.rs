mod cli;
mod error;
mod subcommands;

use clap::StructOpt;
use cli::{Ferium, ProfileSubCommands, SubCommands};
use error::{Error, Result};
use ferinth::Ferinth;
use furse::Furse;
use libium::config;
use tokio::{
	fs::{create_dir_all, remove_dir_all},
	io::AsyncReadExt,
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

async fn actual_main() -> Result<()> {
	// Get the command to execute from Clap
	// This also displays the help page or version
	let cli_app = Ferium::parse();

	// Check for an internet connection
	if online::check(Some(1)).await.is_err() {
		// If it takes more than 1 second
		// show that we're checking the internet connection
		// and check for 4 more seconds
		eprint!("Checking internet connection... ");
		match online::check(Some(4)).await {
			Ok(_) => println!("✓"),
			Err(_) => {
				return Err(Error::Quit(
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
	let mut config_file = config::get_config_file(config::config_file_path()).await?;
	let mut config_file_contents = String::new();
	config_file
		.read_to_string(&mut config_file_contents)
		.await?;
	// Deserialise `config_file` to a config
	let mut config: config::structs::Config = match serde_json::from_str(&config_file_contents) {
		Ok(config) => config,
		Err(err) => {
			return Err(Error::QuitFormatted(format!(
				"Error decoding configuration file, {} at {:?} {}:{}",
				// Error name
				Error::JSONError(err.classify()),
				// File path so that users can find it
				config::config_file_path(),
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
		subcommands::profile::create::create(
			&mut config,
			game_version,
			force_game_version,
			mod_loader,
			name,
			output_dir,
		)
		.await?;

		// Update config file with new values and quit
		config::write_config(&mut config_file, &config).await?;
		return Ok(());
	}

	// Get a mutable reference to the active profile
	let profile = if let Some(profile) = config.profiles.get_mut(config.active_profile) {
		profile
	} else {
		if config.profiles.is_empty() {
			return Err(Error::Quit (
				"There are no profiles configured. Add a profile to your config using `ferium profile create`"
			));
		}
		// Default to first profile if index is set incorrectly
		config.active_profile = 0;
		config::write_config(&mut config_file, &config).await?;
		return Err(Error::Quit(
			"Active profile index points to a non existent profile. Switched to first profile",
		));
	};

	// Run function(s) based on the sub(sub)command to be executed
	match cli_app.subcommand {
		SubCommands::AddModrinth { project_id } => {
			let project = subcommands::add::modrinth(&modrinth, project_id, profile).await?;
			println!("Added {}", project.title);
		},
		SubCommands::AddGithub { owner, name } => {
			let repo = subcommands::add::github(github.repos(owner, name), profile).await?;
			println!("Added {}", repo.name);
		},
		SubCommands::AddCurseforge { project_id } => {
			let project = subcommands::add::curseforge(&curseforge, project_id, profile).await?;
			println!("Added {}", project.name);
		},
		SubCommands::List { verbose } => {
			check_empty_profile(profile)?;
			for mod_ in &profile.mods {
				if verbose {
					match mod_ {
						config::structs::Mod::CurseForgeProject { project_id, .. } => {
							subcommands::list::curseforge(&curseforge, *project_id).await
						},
						config::structs::Mod::ModrinthProject { project_id, .. } => {
							subcommands::list::modrinth(&modrinth, project_id).await
						},
						config::structs::Mod::GitHubRepository { full_name, .. } => {
							subcommands::list::github(&github, full_name).await
						},
					}?;
				} else {
					println!("{}", mod_.name());
				}
			}
		},
		SubCommands::Profile { subcommand } => match subcommand {
			ProfileSubCommands::Configure {
				game_version,
				mod_loader,
				name,
				output_dir,
			} => {
				subcommands::profile::configure::configure(
					profile,
					game_version,
					mod_loader,
					name,
					output_dir,
				)
				.await?;
			},
			// This must have been checked earlier before getting the profile
			ProfileSubCommands::Create { .. } => unreachable!(),
			ProfileSubCommands::Delete { profile_name } => {
				subcommands::profile::delete::delete(&mut config, profile_name)?;
			},
			ProfileSubCommands::List => subcommands::profile::list::list(&config),
		},
		SubCommands::Remove { mod_names } => {
			check_empty_profile(profile)?;
			subcommands::remove::remove(profile, mod_names)?;
		},
		SubCommands::Switch { profile_name } => {
			subcommands::switch::switch(&mut config, profile_name)?;
		},
		SubCommands::Upgrade { no_patch_check } => {
			check_empty_profile(profile)?;
			// Empty the mods directory
			let _ = remove_dir_all(&profile.output_dir).await;
			create_dir_all(&profile.output_dir).await?;
			for mod_ in &profile.mods {
				use libium::config::structs::Mod;
				if let (Err(err), name) = match mod_ {
					Mod::CurseForgeProject { name, project_id } => (
						subcommands::upgrade::curseforge(
							&curseforge,
							profile,
							*project_id,
							no_patch_check,
						)
						.await,
						name,
					),
					Mod::ModrinthProject { name, project_id } => (
						subcommands::upgrade::modrinth(
							&modrinth,
							profile,
							project_id,
							no_patch_check,
						)
						.await,
						name,
					),
					Mod::GitHubRepository { name, full_name } => (
						subcommands::upgrade::github(
							&github.repos(&full_name.0, &full_name.1),
							profile,
						)
						.await,
						name,
					),
				} {
					println!("Could not download {} due to {}", name, err);
				}
			}
		},
	};

	// Update config file with new values
	config::write_config(&mut config_file, &config).await?;

	Ok(())
}

/// Check if `profile` is empty, and if so return an error
fn check_empty_profile(profile: &config::structs::Profile) -> Result<()> {
	if profile.mods.is_empty() {
		Err(Error::EmptyConfigFile)
	} else {
		Ok(())
	}
}
