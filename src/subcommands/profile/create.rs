use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use ferinth::Ferinth;
use libium::{config, file_picker, misc};
use std::path::PathBuf;

pub async fn create(
	modrinth: &Ferinth,
	config: &mut config::structs::Config,
	game_version: Option<String>,
	force_game_version: bool,
	mod_loader: Option<config::structs::ModLoaders>,
	name: Option<String>,
	output_dir: Option<PathBuf>,
) -> Result<()> {
	match (game_version, mod_loader, name, output_dir) {
		(Some(game_version), Some(mod_loader), Some(name), Some(output_dir)) => {
			// If force game version is false
			if !force_game_version {
				// And if the game_version provided does not exist
				if !modrinth
					.list_game_versions()
					.await?
					.iter()
					.any(|version| version.version == game_version)
				{
					// Then error out
					bail!("The game version {} does not exist", game_version);
				}
			}
			// Check that there isn't already a profile with the same name
			for profile in &config.profiles {
				if profile.name == name {
					bail!("A profile with name {} already exists", name);
				}
			}
			// Check that the output_dir isn't relative
			if !output_dir.is_absolute() {
				bail!("The provided output directory is not absolute, i.e. it is a relative path")
			}
			config.profiles.push(config::structs::Profile {
				name,
				output_dir,
				game_version,
				mod_loader,
				mods: Vec::new(),
			}); // Create profile
		},
		(None, None, None, None) => {
			// Create profile using a UI
			println!("Please enter the details for the new profile");

			// Let user pick mods directory
			let mut selected_mods_dir = misc::get_mods_dir();
			println!("The default mods directory is {:?}", selected_mods_dir);
			if Confirm::with_theme(&ColorfulTheme::default())
				.with_prompt("Would you like to specify a custom mods directory?")
				.interact()?
			{
				if let Some(dir) = file_picker::pick_folder(&selected_mods_dir).await {
					selected_mods_dir = dir;
				};
			}

			let mut name = String::new();
			let mut prompt = true;
			while prompt {
				name = Input::with_theme(&ColorfulTheme::default())
					.with_prompt("What should this profile be called?")
					.interact_text()?;

				prompt = false;
				for profile in &config.profiles {
					if profile.name == name {
						println!("A profile with name {} already exists!", name);
						prompt = true;
					}
				}
			}

			// Let user pick Minecraft version
			let mut latest_versions: Vec<String> = misc::get_major_mc_versions(10).await?;
			println!();
			let selected_version = Select::with_theme(&ColorfulTheme::default())
				.with_prompt("Which version of Minecraft do you play?")
				.items(&latest_versions)
				.default(0)
				.interact()?;
			let selected_version = latest_versions.swap_remove(selected_version);

			// Let user pick mod loader
			let mod_loaders = ["Fabric", "Forge"];
			let selected_loader = if Select::with_theme(&ColorfulTheme::default())
				.with_prompt("Which mod loader do you use?")
				.items(&mod_loaders)
				.interact()? == 0
			{
				config::structs::ModLoaders::Fabric
			} else {
				config::structs::ModLoaders::Forge
			};
			config.profiles.push(config::structs::Profile {
				name,
				output_dir: selected_mods_dir,
				mods: Vec::new(),
				game_version: selected_version,
				mod_loader: selected_loader,
			});
		},
		// Either all or none of these options should exist
		// TODO: make this into a group in the Clap app
		_ => {
			bail!("Provide all four arguments to create a profile using options",)
		},
	}

	config.active_profile = config.profiles.len() - 1; // Make created profile active
	Ok(())
}
