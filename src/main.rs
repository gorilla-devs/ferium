mod cli;
mod subcommands;

use anyhow::{bail, Result};
use clap::StructOpt;
use cli::{Ferium, ProfileSubCommands, SubCommands};
use colored::{ColoredString, Colorize};
use ferinth::Ferinth;
use furse::Furse;
use lazy_static::lazy_static;
use libium::{add, config, upgrade};
use tokio::{
	fs::{create_dir_all, remove_dir_all},
	io::AsyncReadExt,
};

lazy_static! {
	pub static ref TICK: ColoredString = "✓".green();
}

#[tokio::main]
async fn main() {
	if let Err(err) = actual_main().await {
		eprintln!("{}", err.to_string().red().bold());
		std::process::exit(1);
	}
}

async fn actual_main() -> Result<()> {
	// This also displays the help page or version automatically
	let cli_app = Ferium::parse();

	// Check for an internet connection
	if online::check(Some(1)).await.is_err() {
		// If it takes more than 1 second
		// show that we're checking the internet connection
		// and check for 4 more seconds
		eprint!("Checking internet connection... ");
		match online::check(Some(4)).await {
			Ok(_) => println!("{}", *TICK),
			Err(_) => bail!("× Ferium requires an internet connection to work"),
		}
	};

	let github = octocrab::instance();
	let modrinth = Ferinth::new();
	let curseforge = Furse::new(env!(
		"CURSEFORGE_API_KEY",
		"A CurseForge API key is required to build. If you don't have one, you can bypass this by setting the variable to a blank string, however anything using the CurseForge API will not work."
	));
	let mut config_file =
		config::get_config_file(cli_app.config_file.unwrap_or_else(config::config_file_path))
			.await?;
	let mut config_file_contents = String::new();
	config_file
		.read_to_string(&mut config_file_contents)
		.await?;
	let mut config: config::structs::Config = serde_json::from_str(&config_file_contents)?;

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
			&modrinth,
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
			bail!("There are no profiles configured. Add a profile to your config using `ferium profile create`")
		}
		// Default to first profile if index is set incorrectly
		config.active_profile = 0;
		config::write_config(&mut config_file, &config).await?;
		bail!("Active profile index points to a non existent profile. Switched to first profile",)
	};

	// Run function(s) based on the sub(sub)command to be executed
	match cli_app.subcommand {
		SubCommands::AddModrinth { project_id } => {
			let project = add::modrinth(&modrinth, project_id, profile).await?;
			println!("Added {}", project.title);
		},
		SubCommands::AddGithub { owner, name } => {
			let repo = add::github(github.repos(owner, name), profile).await?;
			println!("Added {}", repo.name);
		},
		SubCommands::AddCurseforge { project_id } => {
			let project = add::curseforge(&curseforge, project_id, profile).await?;
			println!("Added {}", project.name);
		},
		SubCommands::List { verbose } => {
			check_empty_profile(profile)?;
			for mod_ in &profile.mods {
				if verbose {
					use config::structs::ModIdentifier;
					match &mod_.identifier {
						ModIdentifier::CurseForgeProject(project_id) => {
							subcommands::list::curseforge(&curseforge, *project_id).await
						},
						ModIdentifier::ModrinthProject(project_id) => {
							subcommands::list::modrinth(&modrinth, project_id).await
						},
						ModIdentifier::GitHubRepository(full_name) => {
							subcommands::list::github(&github, full_name).await
						},
					}?;
				} else {
					println!("{}", mod_.name);
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
			let mut error = false;
			for mod_ in &profile.mods {
				use libium::config::structs::ModIdentifier;
				let result = match &mod_.identifier {
					ModIdentifier::CurseForgeProject(project_id) => {
						match upgrade::curseforge(&curseforge, profile, *project_id, no_patch_check)
							.await
						{
							Ok(file) => Ok(file.file_name),
							Err(err) => Err(err),
						}
					},
					ModIdentifier::ModrinthProject(project_id) => {
						match upgrade::modrinth(&modrinth, profile, project_id, no_patch_check)
							.await
						{
							Ok(version) => Ok(version.files[0].filename.clone()),
							Err(err) => Err(err),
						}
					},
					ModIdentifier::GitHubRepository(full_name) => {
						match upgrade::github(&github.repos(&full_name.0, &full_name.1), profile)
							.await
						{
							Ok(asset) => Ok(asset.name),
							Err(err) => Err(err),
						}
					},
				};
				match result {
					Ok(file_name) => {
						println!(
							"{} {:40}{}",
							*TICK,
							mod_.name,
							format!("({})", file_name).dimmed()
						);
					},
					Err(err) => {
						eprintln!("{}", format!("× {:40}{}", mod_.name, err).red());
						error = true;
					},
				}
			}
			if error {
				bail!("\nSome mods were not successfully downloaded")
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
		bail!("Your currently selected profile is empty! Run `ferium help` to see how to add mods");
	}
	Ok(())
}
